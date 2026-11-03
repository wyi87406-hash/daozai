//! 同心额度环、账号选择及账号设置。
use crate::{app::{App,Cmd},extras::{setting_row,settings_label,settings_card},paint::{Card,H},quota::{self,Accent,Account,Config,Provider,Snapshot},theme::{self,W}};
use eframe::egui::{self,Color32};
use std::collections::{HashMap,HashSet};
use chrono::Datelike;

#[derive(Default)]
pub(crate) struct State {
    pub config:Config,pub rows:HashMap<String,Result<Snapshot,String>>,pub busy:HashSet<String>,
    pub provider:Provider,pub alias:String,pub source:String,pub note:String,
    pub login:Option<quota::grok::Login>,
    pub gemini_connecting:Option<quota::gemini::Connect>,pub gemini_email:String,pub gemini_cookie:String,
    pub deepseek_connecting:Option<quota::deepseek::Connect>,pub deepseek_key:String,
}
impl State {
    pub fn load()->Self {Self{config:if cfg!(test){Config::default()}else{Config::load()},..Default::default()}}
    pub fn receive(&mut self,event:quota::Event){self.busy.remove(&event.id);if self.config.accounts.iter().any(|a|a.id==event.id && a.enabled){self.rows.insert(event.id,event.result);}}
}
fn accent_color(accent:Accent,provider:Provider)->Color32{
    let accent=if accent==Accent::Brand{match provider{Provider::Codex=>Accent::Violet,Provider::Claude=>Accent::Orange,Provider::Kimi=>Accent::Blue,Provider::GrokBot=>Accent::Green,Provider::Gemini=>Accent::Cyan,Provider::DeepSeek=>Accent::Blue}}else{accent};
    let (r,g,b)=match accent{Accent::Brand|Accent::Violet=>(142,67,255),Accent::Orange=>(255,116,26),Accent::Blue=>(40,126,255),Accent::Cyan=>(0,213,242),Accent::Pink=>(255,43,132),Accent::Gold=>(255,190,0),Accent::Green=>(52,221,145)};
    Color32::from_rgb(r,g,b)
}
fn accent_colors(config:&Config,a:&Account)->[Color32;2]{
    let main=accent_color(config.colors.get(&a.id).copied().unwrap_or_default(),a.provider);
    [main,crate::paint::mix(main,Color32::WHITE,0.16)]
}
fn color_picker(ui:&mut egui::Ui,provider:Provider,value:&mut Accent){
    ui.horizontal(|ui|{
        ui.label(egui::RichText::new("颜色").size(12.0).color(theme::GRAY));
        if ui.selectable_label(*value==Accent::Brand,egui::RichText::new("默认").size(12.0)).on_hover_text("跟随厂商").clicked(){*value=Accent::Brand;}
        for accent in Accent::ALL.into_iter().skip(1){
            let (rect,response)=ui.allocate_exact_size(egui::vec2(22.0,25.0),egui::Sense::click());
            let on=*value==accent;
            if response.on_hover_text(accent.label()).clicked(){*value=accent;}
            if on{ui.painter().circle_stroke(rect.center(),10.0,egui::Stroke::new(1.5,Color32::WHITE));}
            ui.painter().circle_filled(rect.center(),7.0,accent_color(accent,provider));
        }
        ui.label(egui::RichText::new(value.label()).size(12.0).color(theme::GRAY));
    });
}
pub(crate) fn bar_tooltip(response:egui::Response,text:String)->egui::Response{
    let ctx=&response.ctx;let id=response.id.with("bar-tooltip");
    if response.hovered()&&!ctx.input(|i|i.pointer.any_down()){
        let now=ctx.input(|i|i.time);
        let since=ctx.data_mut(|data|*data.get_temp_mut_or_insert_with(id,||now));
        if now-since>=0.5{
            egui::Area::new(id).order(egui::Order::Tooltip).fixed_pos(response.rect.left_bottom()+egui::vec2(0.0,4.0)).interactable(false).show(ctx,|ui|{
                egui::Frame::popup(ui.style()).show(ui,|ui|{ui.label(&text);});
            });
        }else{ctx.request_repaint_after_secs((0.5-(now-since))as f32);}
    }else{ctx.data_mut(|data|data.remove::<f64>(id));}
    response
}
fn logo_texture(ctx:&egui::Context,provider:Provider,pixels:u32)->Option<egui::TextureHandle>{
    let key=egui::Id::new(("quota-logo",provider.name(),pixels));
    if let Some(texture)=ctx.data_mut(|data|data.get_temp::<egui::TextureHandle>(key)){return Some(texture);}
    let bytes:&[u8]=match provider{Provider::Codex=>include_bytes!("../assets/ai-icons/codex.svg"),Provider::Claude=>include_bytes!("../assets/ai-icons/claude.svg"),Provider::Kimi=>include_bytes!("../assets/ai-icons/kimi.svg"),Provider::GrokBot=>include_bytes!("../assets/ai-icons/grokbot.svg"),Provider::Gemini=>include_bytes!("../assets/ai-icons/gemini.svg"),Provider::DeepSeek=>include_bytes!("../assets/ai-icons/deepseek.svg")};
    let mut image=egui_extras::image::load_svg_bytes_with_size(bytes,Some(egui::load::SizeHint::Size(pixels,pixels))).ok()?;
    // 白色标识使用矢量覆盖率，避免透明边缘再次变暗。
    for pixel in &mut image.pixels{*pixel=Color32::from_white_alpha(pixel.a());}
    let texture=ctx.load_texture(format!("quota-logo-{}-{pixels}",provider.name()),image,egui::TextureOptions::LINEAR);
    ctx.data_mut(|data|data.insert_temp(key,texture.clone()));Some(texture)
}
fn logo(c:&Card,provider:Provider,x:f32,y:f32,size:f32){
    let ctx=c.ui.ctx();let ppp=ctx.pixels_per_point();let center=c.pos(x,y);
    let pixels=(size*c.s*ppp).round().max(1.0) as u32;let half=pixels as f32/2.0;
    let min=egui::pos2((center.x*ppp-half).round()/ppp,(center.y*ppp-half).round()/ppp);
    let rect=egui::Rect::from_min_size(min,egui::Vec2::splat(pixels as f32/ppp));
    if let Some(texture)=logo_texture(ctx,provider,pixels){c.painter.image(texture.id(),rect,egui::Rect::from_min_max(egui::pos2(0.0,0.0),egui::pos2(1.0,1.0)),Color32::WHITE);}
}
fn ring(c:&Card,a:&Account,snapshot:Option<&Snapshot>,x:f32,y:f32,size:f32,colors:[Color32;2]){
    let windows=snapshot.map(Snapshot::rings).unwrap_or_default();let dual=windows.len()>1;
    let stroke=if dual{if size<35.0{2.2}else{6.5}}else{(size*0.102).clamp(3.0,6.5)};
    let radius=size/2.0-stroke/2.0-1.0;
    let gap=if size<35.0{1.5}else{3.0};
    let start=-std::f32::consts::FRAC_PI_2;
    for i in 0..if dual{2}else{1}{
        let r=radius-i as f32*(stroke+gap);
        if let Some(w)=windows.get(i){
            let col=colors[i];let dark=crate::paint::mix(col,Color32::BLACK,0.8);
            if w.remaining>=100.0{c.ring(x,y,r,stroke,col);}else if w.remaining<=0.0{c.ring(x,y,r,stroke,dark);}else{
                let end=start+std::f32::consts::TAU*w.remaining/100.0;
                c.arc(x,y,r,end,start+std::f32::consts::TAU,stroke,dark);
                c.arc(x,y,r,start,end,stroke,col);
                for angle in [start,end]{c.circle(x+r*angle.cos(),y+r*angle.sin(),stroke/2.0,col);}
            }
        }
    }
    if size<35.0 {logo(c,a.provider,x,y,if dual{10.0}else{12.0});}else if dual {logo(c,a.provider,x,y,24.0);}else if let Some(w)=windows.first(){
        if let Some(text)=&w.text{
            let fsize=if text.len()>6{16.0}else{20.0};
            let value=c.fit(text,fsize,W::Bold,size*1.05);c.centered_text(x,y,&value,fsize,W::Bold,theme::WHITE,H::Center);
        }else{
            let value=format!("{:.0}",w.remaining);let large=if value.len()>2{22.0}else{28.0};
            let number_width=c.text_w(&value,large,W::Bold);let width=number_width+c.text_w("%",11.0,W::Semibold)+2.0;
            let left=x-width/2.0;c.centered_text(left,y,&value,large,W::Bold,theme::WHITE,H::Left);
            c.centered_text(left+number_width+2.0,y+3.0,"%",11.0,W::Semibold,theme::GRAY,H::Left);
        }
    }else{c.centered_text(x,y,"—",24.0,W::Semibold,theme::DIM,H::Center);}
}
fn reset_text(at:Option<i64>,now:&chrono::DateTime<chrono::Local>)->String {
    let Some(at)=at.and_then(|at|chrono::DateTime::from_timestamp(at,0)) else{return "重置时间未知".into();};
    let at=at.with_timezone(&chrono::Local);
    if at.timestamp()<=now.timestamp(){return "等待刷新".into();}
    let format=if at.date_naive()==now.date_naive(){"今天 %H:%M 重置"}else if at.year()==now.year(){"%-m月%-d日 %H:%M 重置"}else{"%Y/%-m/%-d %H:%M 重置"};
    at.format(format).to_string()
}
impl App {
    pub(crate) fn quota_tick(&mut self){
        let result=self.quota.gemini_connecting.as_ref().and_then(|connect|connect.poll());
        if let Some(result)=result{
            self.quota.gemini_connecting=None;
            match result{Ok(connected)=>{let id=self.quota.config.connect_gemini(connected.slot,connected.label);self.quota.rows.insert(id,Ok(connected.snapshot));self.quota.config.save();self.quota.gemini_cookie.clear();self.quota.note="Gemini 已连接".into();self.pending.push(Cmd::QuotaConnected);},Err(error)=>self.quota.note=error}
        }
        let result=self.quota.deepseek_connecting.as_ref().and_then(|connect|connect.poll());
        if let Some(result)=result{
            self.quota.deepseek_connecting=None;
            match result{Ok(connected)=>{let id=self.quota.config.connect_deepseek(connected.slot,connected.label);self.quota.rows.insert(id,Ok(connected.snapshot));self.quota.config.save();self.quota.deepseek_key.clear();self.quota.note="DeepSeek 已添加".into();self.pending.push(Cmd::QuotaConnected);},Err(error)=>self.quota.note=error}
        }
        let result=self.quota.login.as_ref().and_then(|login|login.poll());
        if let Some(result)=result{
            self.quota.login=None;
            match result{Ok(connected)=>{self.quota.config.connect_grok(connected.slot,connected.label);self.quota.config.save();self.quota.note="Grok Bot 已连接".into();self.pending.push(Cmd::QuotaConnected);},Err(error)=>self.quota.note=error}
        }
    }
    fn quota_login(&mut self,alias:String){
        if self.quota.login.is_some(){return;}
        match quota::grok::Login::start(alias){Ok((login,url))=>{self.quota.login=Some(login);self.quota.note="请在浏览器中完成登录，账号会自动添加".into();self.pending.push(Cmd::OpenUrl(url));},Err(error)=>self.quota.note=error}
    }
    pub fn quota_config(&self)->&Config{&self.quota.config}
    pub fn quota_initialize(&mut self){if !crate::model::config_dir().join("ai-quota.json").exists(){self.quota.config.discover();self.quota.config.save();}}
    pub fn quota_pending(&mut self,accounts:&[Account]){self.quota.busy.extend(accounts.iter().map(|a|a.id.clone()));}
    pub fn quota_event(&mut self,event:quota::Event){self.quota.receive(event);}
    pub fn quota_set_source(&mut self,path:std::path::PathBuf){self.quota.source=path.to_string_lossy().into_owned();}
    pub(crate) fn quota_bar_indices(&self)->Vec<usize>{
        if !self.quota.config.enabled || !self.quota.config.compact || !self.settings.visible_pages[6] || self.settings.minimal || self.t<self.notification_bell_until{return vec![];}
        self.quota.config.pinned_indices()
    }
    pub(crate) fn draw_quota_bar(&mut self,c:&Card,cx:f32,layout:&crate::bar_layout::Layout){
        let indices=self.quota_bar_indices();let mut chosen=None;
        for (position,i)in indices.iter().take(3).enumerate(){
            let alpha=layout.pet_clearance(crate::bar_layout::QUOTA+position);
            let faded=c.faded(alpha);let faded=if alpha<0.8{faded.ghost()}else{faded};let c=&faded;
            let a=&self.quota.config.accounts[*i];let x=cx+layout.x[crate::bar_layout::QUOTA+position].unwrap();
            let snapshot=self.quota.rows.get(&a.id).and_then(|r|r.as_ref().ok());ring(c,a,snapshot,x,15.0,27.0,accent_colors(&self.quota.config,a));
            let mut same=self.quota.config.accounts.iter().filter(|other|other.provider==a.provider && other.enabled);
            if same.clone().count()>1{let n=same.position(|other|other.id==a.id).unwrap_or(0)+1;c.circle(x+10.0,24.0,4.5,Color32::from_gray(225));c.text(x+10.0,26.8,&n.to_string(),6.5,W::Bold,Color32::BLACK,H::Center);}
            let summary=snapshot.map(|s|s.rings().iter().map(|w|format!("{}剩余 {:.0}%",w.label,w.remaining)).collect::<Vec<_>>().join(" · ")).unwrap_or_else(||if self.quota.busy.contains(&a.id){"正在更新".into()}else{"额度暂不可用".into()});
            if bar_tooltip(c.hit(x-14.0,1.0,28.0,28.0,("quota-pin",&a.id)),format!("{} · {}\n{}",a.provider.name(),a.alias,summary)).clicked(){chosen=Some(a.id.clone());}
        }
        if let Some(x)=layout.x[crate::bar_layout::MORE] {let alpha=layout.pet_clearance(crate::bar_layout::MORE);let faded=c.faded(alpha);let faded=if alpha<0.8{faded.ghost()}else{faded};let c=&faded;let x=cx+x;c.text(x,20.0,&format!("+{}",indices.len()-3),8.0,W::Regular,theme::GRAY,H::Center);if bar_tooltip(c.hit(x-11.0,3.0,22.0,25.0,"quota-more"),"查看全部账号".into()).clicked(){self.show_page(6);}}
        if let Some(id)=chosen{self.quota.config.selected=id;self.quota.config.save();self.show_page(6);}
    }
    pub(crate) fn page_quota(&mut self,c:&Card){
        if c.hit(303.0,8.0,32.0,21.0,"quota-refresh").on_hover_text("刷新额度").clicked(){self.pending.push(Cmd::QuotaRefresh);}
        c.text(319.0,23.0,if self.quota.busy.is_empty(){"刷新"}else{"更新中"},10.5,W::Regular,theme::GRAY,H::Center);
        let Some(selected)=self.quota.config.selected_index() else{
            c.text(43.0,61.0,"还没有添加 AI 账号",13.0,W::Bold,theme::WHITE,H::Left);
            c.text(43.0,80.0,"连接 AI 账号，查看剩余额度",10.5,W::Regular,theme::GRAY,H::Left);
            let(r,b)=crate::paint::capsule(c,43.0,92.0,94.0,24.0,"add-quota-account",theme::BTN,theme::BTN_HOVER);b.text(90.0,108.0,"添加账号",10.0,W::Semibold,theme::WHITE,H::Center);if r.clicked(){self.settings_tab=11;self.open_settings();}return;
        };
        let indices:Vec<_>=self.quota.config.accounts.iter().enumerate().filter_map(|(i,a)|a.enabled.then_some(i)).collect();
        let pos=indices.iter().position(|i|*i==selected).unwrap_or(0);let start=pos/3*3;let mut picked=None;
        if indices.len()>3{for (x,offset,label)in [(42.0,-1isize,"‹"),(357.0,1,"›")]{if c.hit(x-7.0,30.0,18.0,19.0,("quota-account-next",offset)).clicked(){picked=Some(indices[(pos as isize+offset).rem_euclid(indices.len() as isize) as usize]);}c.text(x+2.0,44.0,label,13.0,W::Regular,theme::GRAY,H::Center);}}
        for (column,index)in indices.iter().skip(start).take(3).enumerate(){let a=&self.quota.config.accounts[*index];let x=59.0+column as f32*99.0;let on=*index==selected;let colors=accent_colors(&self.quota.config,a);let color=colors[0];let(r,b)=crate::paint::capsule(c,x,30.0,94.0,21.0,&format!("quota-account-{}",a.id),if on{Color32::from_rgb(color.r()/5,color.g()/5,color.b()/5)}else{Color32::BLACK},theme::BTN_HOVER);let text=b.fit(&a.alias,10.5,W::Regular,84.0);b.text(x+47.0,45.0,&text,10.5,W::Regular,if on{colors[1]}else{theme::GRAY},H::Center);if r.on_hover_text(format!("{} · {}",a.provider.name(),a.alias)).clicked(){picked=Some(*index);}}
        if let Some(i)=picked{self.quota.config.selected=self.quota.config.accounts[i].id.clone();self.quota.config.save();}
        let a=&self.quota.config.accounts[selected];let result=self.quota.rows.get(&a.id);let snapshot=result.and_then(|r|r.as_ref().ok());
        let colors=accent_colors(&self.quota.config,a);ring(c,a,snapshot,103.0,89.0,70.0,colors);
        let raw_plan=snapshot.map(|s|s.plan.as_str()).unwrap_or_default();let plan=match raw_plan{"pro"|"prolite"|"pro_lite"=>"Pro","plus"=>"Plus","free"=>"Free","team"|"business"=>"Business","enterprise"=>"Enterprise",other=>other};let label=if plan.is_empty(){a.provider.name().to_owned()}else{format!("{} · {}",a.provider.name(),plan)};
        c.text(157.0,64.0,&c.fit(&label,12.0,W::Semibold,145.0),12.0,W::Semibold,theme::WHITE,H::Left);
        if let Some(s)=snapshot{
            let windows=s.rings();let now=chrono::Local::now();
            if windows.len()==1{
                let w=windows[0];c.text(157.0,87.0,&format!("{}剩余",w.label),12.0,W::Semibold,theme::WHITE,H::Left);
                c.text(157.0,108.0,&reset_text(w.reset,&now),11.0,W::Regular,theme::GRAY,H::Left);
            }else{
                c.text(355.0,64.0,"剩余",10.5,W::Regular,theme::GRAY,H::Right);
                for (i,w)in windows.iter().enumerate(){
                    let y=79.0+i as f32*32.0;c.circle(159.0,y-4.0,2.5,colors[i]);
                    c.text(168.0,y,&w.label,11.5,W::Semibold,theme::WHITE,H::Left);
                    c.text(168.0,y+14.0,&reset_text(w.reset,&now),11.0,W::Regular,theme::GRAY,H::Left);
                    c.centered_text(355.0,y+4.0,&format!("{:.0}%",w.remaining),22.0,W::Semibold,theme::WHITE,H::Right);
                }
            }
        }else{let message=if self.quota.busy.contains(&a.id){"正在更新额度…"}else{result.and_then(|r|r.as_ref().err()).map(String::as_str).unwrap_or("等待更新额度")};for(i,line)in c.wrap(message,11.0,W::Regular,197.0,2).iter().enumerate(){c.text(157.0,87.0+i as f32*17.0,line,11.0,W::Regular,theme::GRAY,H::Left);}}
    }
    pub(crate) fn quota_settings(&mut self,ui:&mut egui::Ui){
        let before=self.quota.config.clone();settings_label(ui,"显示");settings_card(ui,|ui|{
            setting_row(ui,"AI 额度","每 3 分钟自动更新，可手动刷新",&mut self.quota.config.enabled);ui.separator();
            setting_row(ui,"收起时显示圆环","常驻账号最多显示 3 个，点击查看详情",&mut self.quota.config.compact);
        });
        settings_label(ui,"账号");let mut remove=None;let mut reconnect=None;let mut gemini_reconnect=None;
        settings_card(ui,|ui|{
            if self.quota.config.accounts.is_empty(){ui.label("还没有添加账号。可识别本机登录，或在下方连接其他账号。");}
            for a in &mut self.quota.config.accounts{ui.push_id(&a.id,|ui|{
                ui.horizontal(|ui|{ui.label(egui::RichText::new(a.provider.name()).strong());ui.add_sized([150.0,25.0],egui::TextEdit::singleline(&mut a.alias).hint_text("账号备注"));if a.provider==Provider::GrokBot && ui.add_enabled(self.quota.login.is_none(),egui::Button::new("重新连接").small()).clicked(){reconnect=Some(a.alias.clone());}if a.provider==Provider::Gemini && ui.add_enabled(self.quota.gemini_connecting.is_none(),egui::Button::new("重新连接").small()).clicked(){gemini_reconnect=Some((a.alias.clone(),a.slot.clone()));}if ui.add_enabled((a.provider!=Provider::Gemini || self.quota.gemini_connecting.is_none()) && (a.provider!=Provider::DeepSeek || self.quota.deepseek_connecting.is_none()),egui::Button::new("移除").small()).clicked(){remove=Some(a.id.clone());}});
                ui.horizontal(|ui|{ui.checkbox(&mut a.enabled,"监控");ui.checkbox(&mut a.pinned,"常驻刘海");if ui.selectable_label(self.quota.config.selected==a.id,"当前账号").clicked(){self.quota.config.selected=a.id.clone();}});
                let mut color=self.quota.config.colors.get(&a.id).copied().unwrap_or_default();color_picker(ui,a.provider,&mut color);if color==Accent::Brand{self.quota.config.colors.remove(&a.id);}else{self.quota.config.colors.insert(a.id.clone(),color);}
                let source=if a.provider==Provider::Gemini{"Gemini 网页订阅 · Windows 加密保存".into()}else if a.provider==Provider::GrokBot{if a.source.as_os_str().is_empty(){"浏览器登录 · Windows 安全存储".into()}else{"Grok Bot 本机登录".into()}}else if a.provider==Provider::DeepSeek{"DeepSeek 开放平台 · Windows 加密保存".into()}else{a.source.to_string_lossy()};
                ui.label(egui::RichText::new(source).size(11.0).color(Color32::from_gray(170)));
                let message=if self.quota.busy.contains(&a.id){"正在更新…".into()}else{match self.quota.rows.get(&a.id){Some(Ok(s))=>format!("已更新 · {} 个额度周期",s.windows.len()),Some(Err(e))=>e.clone(),None=>"等待更新".into()}};ui.label(egui::RichText::new(message).size(11.0).color(Color32::from_gray(170)));ui.add_space(6.0);
            });}
            ui.horizontal(|ui|{if ui.button("识别本机账号").clicked(){self.quota.note=self.quota.config.discover().message();}if ui.add_enabled(self.quota.busy.is_empty(),egui::Button::new("刷新额度")).clicked(){self.pending.push(Cmd::QuotaRefresh);}});
            ui.label(egui::RichText::new("本机识别支持 Codex、Claude 桌面版与 Code、Kimi Code、Grok Bot；Gemini 请连接浏览器。").size(12.0).color(Color32::from_gray(180)));
        });
        if let Some(id)=remove{if let Some(account)=self.quota.config.accounts.iter().find(|a|a.id==id){match account.provider{Provider::GrokBot=>{quota::grok::forget(account);self.quota.login=None;},Provider::Gemini=>quota::gemini::forget(account),Provider::DeepSeek=>quota::deepseek::forget(account),_=>{}}}self.quota.config.accounts.retain(|a|a.id!=id);self.quota.config.colors.remove(&id);self.quota.rows.remove(&id);self.quota.busy.remove(&id);if self.quota.config.selected==id{self.quota.config.selected=self.quota.config.accounts.first().map(|a|a.id.clone()).unwrap_or_default();}}
        if let Some(alias)=reconnect{self.quota_login(alias);}
        if let Some((alias,slot))=gemini_reconnect{self.quota.gemini_connecting=Some(quota::gemini::Connect::start(alias,String::new(),None,slot));}
        settings_label(ui,"添加其他账号");settings_card(ui,|ui|{
            ui.horizontal(|ui|{ui.label("服务");egui::ComboBox::from_id_salt("quota-provider").selected_text(self.quota.provider.name()).show_ui(ui,|ui|{for p in Provider::ALL{ui.selectable_value(&mut self.quota.provider,p,p.name());}});});
            ui.horizontal(|ui|{ui.label("备注");ui.add_sized([ui.available_width(),27.0],egui::TextEdit::singleline(&mut self.quota.alias).hint_text("例如：个人、工作、备用"));});
            if self.quota.provider==Provider::Gemini{
                ui.label(egui::RichText::new("读取 Gemini 网页的 5 小时与每周额度。").size(12.0).color(Color32::from_gray(180)));
                ui.horizontal(|ui|{if ui.button("打开 Gemini 用量").clicked(){self.pending.push(Cmd::OpenUrl("https://gemini.google.com/usage".into()));}if ui.add_enabled(self.quota.gemini_connecting.is_none(),egui::Button::new("连接当前浏览器")).clicked(){self.quota.gemini_connecting=Some(quota::gemini::Connect::start(self.quota.alias.clone(),self.quota.gemini_email.clone(),None,None));}});
                ui.label(egui::RichText::new("浏览器连接需要 Kimi 浏览器扩展；连接后可独立刷新。").size(12.0).color(Color32::from_gray(180)));
                ui.horizontal(|ui|{ui.label("Google 邮箱");ui.add_sized([ui.available_width(),27.0],egui::TextEdit::singleline(&mut self.quota.gemini_email).hint_text("多账号时填写要连接的邮箱"));});
                ui.collapsing("使用 Cookie 连接",|ui|{
                    ui.label("在 Gemini 用量页按 F12，打开网络，刷新后找到 executeQuery 请求，复制 Cookie 请求头。");
                    ui.add_sized([ui.available_width(),27.0],egui::TextEdit::singleline(&mut self.quota.gemini_cookie).password(true).hint_text("粘贴 Cookie 请求头"));
                    if ui.add_enabled(self.quota.gemini_connecting.is_none() && !self.quota.gemini_cookie.trim().is_empty(),egui::Button::new("验证并连接")).clicked(){self.quota.gemini_connecting=Some(quota::gemini::Connect::start(self.quota.alias.clone(),self.quota.gemini_email.clone(),Some(self.quota.gemini_cookie.clone()),None));}
                });
            }else if self.quota.provider==Provider::GrokBot{
                ui.horizontal(|ui|{
                    if ui.add_enabled(self.quota.login.is_none(),egui::Button::new("浏览器登录")).clicked(){self.quota_login(self.quota.alias.clone());}
                    if ui.button("从本机导入").clicked(){self.quota.note=match quota::grok::native_path(){Some(path)=>match self.quota.config.import_grok(&path,&self.quota.alias){Ok(n)=>format!("新增 {n} 个 Grok Bot 账号"),Err(e)=>e},None=>"请先安装 Grok Bot 并登录，或使用浏览器连接".into()};}
                });
                ui.collapsing("导入其他登录文件",|ui|{self.quota_file_form(ui);});
                ui.label(egui::RichText::new("使用 Grok Bot 对应的 Cursor 账号登录，浏览器授权支持自动续期。").size(12.0).color(Color32::from_gray(180)));
            }else if self.quota.provider==Provider::DeepSeek{
                ui.label(egui::RichText::new("填写 DeepSeek 开放平台的 API Key，用于查询账户余额。").size(12.0).color(Color32::from_gray(180)));
                ui.horizontal(|ui|{ui.label("API Key");ui.add_sized([ui.available_width(),27.0],egui::TextEdit::singleline(&mut self.quota.deepseek_key).password(true).hint_text("sk-…"));});
                if ui.add_enabled(self.quota.deepseek_connecting.is_none() && !self.quota.deepseek_key.trim().is_empty(),egui::Button::new("验证并添加")).clicked(){self.quota.deepseek_connecting=Some(quota::deepseek::Connect::start(self.quota.alias.clone(),self.quota.deepseek_key.clone()));}
                ui.label(egui::RichText::new("API Key 在 platform.deepseek.com 的「API keys」页面创建；仅保存在本机用于查询余额。").size(12.0).color(Color32::from_gray(180)));
            }else{
                self.quota_file_form(ui);
                ui.label(egui::RichText::new("各账号使用各自的登录文件；移除只取消监控。登录失效时，请在对应工具中重新登录。").size(12.0).color(Color32::from_gray(180)));
            }
        });
        if self.quota.login.is_some(){ui.horizontal(|ui|{ui.spinner();ui.label("等待浏览器登录…");if ui.button("取消登录").clicked(){self.quota.login=None;self.quota.note="登录已取消".into();}});ui.ctx().request_repaint_after(std::time::Duration::from_millis(100));}
        if self.quota.gemini_connecting.is_some(){ui.horizontal(|ui|{ui.spinner();ui.label("正在连接 Gemini…");});ui.ctx().request_repaint_after(std::time::Duration::from_millis(100));}
        if self.quota.deepseek_connecting.is_some(){ui.horizontal(|ui|{ui.spinner();ui.label("正在验证 DeepSeek…");});ui.ctx().request_repaint_after(std::time::Duration::from_millis(100));}
        if !self.quota.note.is_empty(){ui.label(egui::RichText::new(&self.quota.note).size(12.0).color(Color32::from_gray(190)));}
        if before!=self.quota.config{self.quota.config.save();}
    }
    fn quota_file_form(&mut self,ui:&mut egui::Ui){
        ui.label("登录 JSON 文件");ui.horizontal(|ui|{ui.add_sized([(ui.available_width()-65.0).max(100.0),27.0],egui::TextEdit::singleline(&mut self.quota.source).hint_text("选择该账号的登录文件"));if ui.button("选择…").clicked(){self.pending.push(Cmd::QuotaChooseFile);}});
        if ui.button("添加账号").clicked(){match self.quota.config.add(self.quota.provider,self.quota.alias.clone(),self.quota.source.trim().into()){Ok(())=>{self.quota.alias.clear();self.quota.source.clear();self.quota.note="账号已添加".into();},Err(e)=>self.quota.note=e}}
    }
    #[cfg(feature="record")]
    pub fn quota_demo(&mut self,mode:&str){
        use serde_json::json;
        self.quota=State::default();let reset=chrono::Utc::now().timestamp()+8200;
        for(i,(provider,alias))in [(Provider::Codex,"个人"),(Provider::Codex,"工作"),(Provider::Claude,"Claude")].into_iter().enumerate(){let id=format!("demo-{i}");self.quota.config.accounts.push(Account{id:id.clone(),provider,alias:alias.into(),source:format!("demo-{i}.json").into(),slot:None,enabled:true,pinned:mode=="multi" || i==0});let payload=if provider==Provider::Codex{json!({"plan_type":if i==0{"pro"}else{"plus"},"rate_limit":{"primary_window":if mode=="single" && i==0{serde_json::Value::Null}else{json!({"used_percent":28+i*15,"limit_window_seconds":18000,"reset_at":reset})},"secondary_window":{"used_percent":56,"limit_window_seconds":604800,"reset_at":reset+200000}}})}else{json!({"five_hour":{"utilization":45,"resets_at":"2030-01-01T12:00:00Z"},"seven_day":{"utilization":27}})};self.quota.rows.insert(id,quota::parse(provider,&payload));}
        self.quota.config.selected="demo-0".into();self.show_page(6);if self.settings_open{self.settings_tab=11;}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    #[test]
    fn quota_logo_matches_screen_pixels_at_each_display_scale(){
        for scale in [1.0,1.25,1.5,1.75,2.0] {
            let ctx=egui::Context::default();egui_extras::install_image_loaders(&ctx);
            for provider in Provider::ALL {for size in [10.0,12.0,20.0,24.0] {
                let raw=egui::RawInput{screen_rect:Some(egui::Rect::from_min_size(egui::Pos2::ZERO,egui::vec2(120.0,80.0))),viewports:std::iter::once((egui::ViewportId::ROOT,egui::ViewportInfo{native_pixels_per_point:Some(scale),..Default::default()})).collect(),..Default::default()};
                let output=ctx.run(raw,|ctx|{egui::CentralPanel::default().frame(egui::Frame::new()).show(ctx,|ui|{
                    let c=Card::new(ui,egui::pos2(0.37,0.21),ui.max_rect());logo(&c,provider,32.25,32.5,size);
                });});
                let mesh=output.shapes.iter().find_map(|s|match &s.shape{egui::Shape::Mesh(mesh)=>Some(mesh),_=>None}).expect("应绘制厂商图标");
                let image=&output.textures_delta.set.iter().find(|(id,_)|*id==mesh.texture_id).expect("不同尺寸应各自生成纹理").1.image;
                let pixels=(size*scale).round() as usize;
                assert_eq!(image.size(),[pixels,pixels],"图标应按屏幕像素生成，厂商={}，缩放={scale}",provider.name());
                for v in &mesh.vertices {
                    assert!((v.pos.x*scale-(v.pos.x*scale).round()).abs()<0.001 && (v.pos.y*scale-(v.pos.y*scale).round()).abs()<0.001,"图标边界应对齐物理像素");
                }
                if let egui::ImageData::Color(image)=image {for pixel in &image.pixels {assert_eq!(*pixel,Color32::from_white_alpha(pixel.a()),"白色标识的透明边缘不应发灰");}}
            }}
        }
    }
    #[test]
    fn quota_reset_uses_local_calendar_dates_and_clock_times(){
        let now=chrono::Local.with_ymd_and_hms(2026,10,31,22,0,0).unwrap();
        for (parts,expected) in [
            ((2026,10,31,23,30),"今天 23:30 重置"),
            ((2026,11,1,9,0),"11月1日 09:00 重置"),
            ((2026,11,3,9,0),"11月3日 09:00 重置"),
            ((2027,1,1,9,0),"2027/1/1 09:00 重置"),
        ] {
            let (year,month,day,hour,minute)=parts;
            let at=chrono::Local.with_ymd_and_hms(year,month,day,hour,minute,0).unwrap().timestamp();
            assert_eq!(reset_text(Some(at),&now),expected);
        }
    }
    #[test]
    fn quota_reset_does_not_invent_missing_or_expired_times(){
        let now=chrono::Local.with_ymd_and_hms(2026,10,31,22,0,0).unwrap();
        assert_eq!(reset_text(None,&now),"重置时间未知");
        assert_eq!(reset_text(Some(i64::MAX),&now),"重置时间未知");
        assert_eq!(reset_text(Some(now.timestamp()),&now),"等待刷新");
        assert_eq!(reset_text(Some(now.timestamp()-1),&now),"等待刷新");
    }
}
