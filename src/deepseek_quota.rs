//! DeepSeek 开放平台余额。用户填写 API Key，凭据由 Windows DPAPI 加密保存。
//! 余额接口：GET https://api.deepseek.com/user/balance（Bearer API Key）。
use super::{Account,Snapshot,Window};
use serde_json::Value;
use std::{fs,io::Read,path::PathBuf,sync::mpsc};

const ENDPOINT:&str="https://api.deepseek.com/user/balance";
const LOGIN_ERROR:&str="DeepSeek API Key 无效或已失效，请重新填写";
const PARSE_ERROR:&str="DeepSeek 额度接口已变化，暂时无法读取";

fn key(raw:&str)->Result<String,String>{
    let raw=raw.trim();
    if raw.is_empty() || raw.len()>256 || !raw.is_ascii() || raw.bytes().any(|b|b<32 || b==127){return Err("请填写有效的 DeepSeek API Key（sk- 开头）".into());}
    Ok(raw.into())
}
fn slot(key:&str)->String{ring::digest::digest(&ring::digest::SHA256,key.as_bytes()).as_ref().iter().map(|b|format!("{b:02x}")).collect()}
fn session_path(slot:&str)->Result<PathBuf,String>{
    if slot.len()!=64 || !slot.bytes().all(|b|b.is_ascii_hexdigit()){return Err(LOGIN_ERROR.into());}
    Ok(crate::model::config_dir().join("credentials").join(format!("deepseek-{slot}.bin")))
}
#[cfg(windows)]
fn protect(raw:&[u8],encrypt:bool)->Result<Vec<u8>,String>{
    use windows::Win32::{Foundation::{HLOCAL,LocalFree},Security::Cryptography::{CryptProtectData,CryptUnprotectData,CRYPT_INTEGER_BLOB,CRYPTPROTECT_UI_FORBIDDEN}};
    let input=CRYPT_INTEGER_BLOB{cbData:raw.len().try_into().map_err(|_|LOGIN_ERROR)?,pbData:raw.as_ptr().cast_mut()};let mut output=CRYPT_INTEGER_BLOB::default();
    unsafe{
        if encrypt{CryptProtectData(&input,None,None,None,None,CRYPTPROTECT_UI_FORBIDDEN,&mut output)}else{CryptUnprotectData(&input,None,None,None,None,CRYPTPROTECT_UI_FORBIDDEN,&mut output)}.map_err(|_|"无法访问 Windows 安全保存的 DeepSeek 凭据")?;
        let bytes=std::slice::from_raw_parts(output.pbData,output.cbData as usize).to_vec();let _=LocalFree(HLOCAL(output.pbData.cast()));Ok(bytes)
    }
}
#[cfg(not(windows))]
fn protect(_raw:&[u8],_encrypt:bool)->Result<Vec<u8>,String>{Err("当前系统暂不支持安全保存 DeepSeek 凭据".into())}

fn save(api_key:&str)->Result<String,String>{
    let s=slot(api_key);let path=session_path(&s)?;
    let raw=serde_json::json!({"api_key":api_key}).to_string();let encrypted=protect(raw.as_bytes(),true)?;
    fs::create_dir_all(path.parent().unwrap()).map_err(|_|"无法保存 DeepSeek 凭据")?;
    crate::model::atomic_write(&path,&encrypted).map_err(|_|"无法保存 DeepSeek 凭据")?;Ok(s)
}
fn load(account:&Account)->Result<String,String>{
    let expected=account.slot.as_deref().ok_or(LOGIN_ERROR)?;let path=session_path(expected)?;
    let raw=fs::read(path).map_err(|_|LOGIN_ERROR)?;let raw=protect(&raw,false)?;
    let value:Value=serde_json::from_slice(&raw).map_err(|_|LOGIN_ERROR)?;
    let api_key=key(value["api_key"].as_str().ok_or(LOGIN_ERROR)?)?;
    if slot(&api_key)!=expected{return Err("DeepSeek 凭据不一致，请重新填写".into());}
    Ok(api_key)
}
pub fn forget(account:&Account){if let Some(path)=account.slot.as_deref().and_then(|s|session_path(s).ok()){let _=fs::remove_file(path);}}

fn parse(value:&Value)->Result<Snapshot,String>{
    if value["is_available"].as_bool()!=Some(true){return Err("DeepSeek 账户当前不可用或没有余额".into());}
    let balance=value["balance_infos"].as_array().and_then(|a|a.first()).ok_or(PARSE_ERROR)?;
    let total=super::number(&balance["total_balance"]).ok_or(PARSE_ERROR)?;
    let currency=balance["currency"].as_str().unwrap_or("CNY");
    let symbol=match currency{"USD"=>"$","CNY"=>"¥",_=>""};
    let text=format!("{symbol}{total:.2}");
    let remaining=if total>0.0{100.0}else{0.0};
    Ok(Snapshot{plan:currency.into(),windows:vec![Window{label:"余额".into(),seconds:None,remaining,reset:None,text:Some(text)}],extra:vec![],at:chrono::Utc::now().timestamp()})
}
fn request(agent:&ureq::Agent,api_key:&str)->Result<Snapshot,(String,u64)>{
    let response=agent.get(ENDPOINT).set("Authorization",&format!("Bearer {api_key}")).set("Accept","application/json").call().map_err(|e|match e{
        ureq::Error::Status(401|403,_)=>(LOGIN_ERROR.into(),0),
        ureq::Error::Status(429,r)=>("DeepSeek 请求过于频繁，稍后自动重试".into(),r.header("Retry-After").and_then(|s|s.parse().ok()).unwrap_or(180).clamp(30,3600)),
        ureq::Error::Status(n,_)=>(format!("DeepSeek 额度服务暂不可用（{n}）"),60),
        _=>("无法连接 DeepSeek，请检查网络或代理".into(),30)
    })?;
    let mut body=String::new();response.into_reader().take(1024*1024).read_to_string(&mut body).map_err(|_|("读取 DeepSeek 额度失败".into(),30))?;
    let value:Value=serde_json::from_str(&body).map_err(|_|("DeepSeek 未返回有效数据".into(),60))?;
    parse(&value).map_err(|e|(e,60))
}
pub fn fetch(agent:&ureq::Agent,account:&Account)->Result<Snapshot,(String,u64)>{
    let api_key=load(account).map_err(|e|(e,0))?;request(agent,&api_key)
}
pub struct Connected {pub slot:String,pub label:String,pub snapshot:Snapshot}
fn connect(alias:String,api_key:String)->Result<Connected,String>{
    let api_key=key(&api_key)?;
    let snapshot=request(&super::http_agent(),&api_key).map_err(|e|e.0)?;
    let slot=save(&api_key)?;
    let label=if alias.trim().is_empty(){"DeepSeek 账号".into()}else{alias.trim().into()};
    Ok(Connected{slot,label,snapshot})
}
pub struct Connect {rx:mpsc::Receiver<Result<Connected,String>>}
impl Connect {
    pub fn start(alias:String,api_key:String)->Self{
        let(tx,rx)=mpsc::channel();std::thread::spawn(move||{let _=tx.send(connect(alias,api_key));});Self{rx}
    }
    pub fn poll(&self)->Option<Result<Connected,String>>{self.rx.try_recv().ok()}
}
