//! 设置、通知队列与岛仔表情。
use crate::app::{App, Cmd, Phase, HUD_TRACK_W, HUD_TRACK_X, HUD_W, PAGE_NAMES};
use crate::paint::{Card, H};
use crate::theme::{self, size, W};
use eframe::egui::{self, Color32, Rect};

#[derive(Clone, Debug)]
pub(crate) struct Alert {
    pub title: String,
    pub body: String,
    pub page: usize,
    pub at: f64,
    pub duration: f64,
    pub bluetooth: Option<(crate::bt::DeviceKind, bool)>,
    pub meeting: Option<(crate::mail::Meeting, String)>,
}
impl Alert {
    pub(crate) fn is_hud(&self)->bool {matches!(self.title.as_str(),"系统音量"|"大写锁定"|"键盘输入语言"|"网络连接"|"电源已连接"|"已切换电池供电"|"蓝牙设备")}
}
fn bluetooth_notice_content(body:&str)->(&str,&str,Option<u8>){
    if let Some(name)=body.strip_suffix(" · 已断开") {return (name,"已断开",None);}
    if body=="已断开连接" {return ("蓝牙设备","已断开",None);}
    if let Some((name,level))=body.rsplit_once(" · 电量 ") {
        if let Some(level)=level.strip_suffix('%').and_then(|s|s.parse::<u8>().ok()).filter(|v|*v<=100){
            return (name,"已连接",Some(level));
        }
    }
    (body,"已连接",None)
}

pub(crate) fn device_icon(c: &Card, kind: crate::bt::DeviceKind, x: f32, y: f32, size: f32) {
    let icon=match kind {
        crate::bt::DeviceKind::Bone | crate::bt::DeviceKind::Earbuds | crate::bt::DeviceKind::Headphones=>egui::include_image!("../assets/device-icons/headphones-round-bold.svg"),
        crate::bt::DeviceKind::Speaker=>egui::include_image!("../assets/device-icons/speaker-bold.svg"),
        crate::bt::DeviceKind::Keyboard=>egui::include_image!("../assets/device-icons/keyboard-bold.svg"),
        crate::bt::DeviceKind::Mouse=>egui::include_image!("../assets/device-icons/mouse-bold.svg"),
        crate::bt::DeviceKind::Gamepad=>egui::include_image!("../assets/device-icons/gamepad-bold.svg"),
        crate::bt::DeviceKind::Phone=>egui::include_image!("../assets/device-icons/smartphone-bold.svg"),
        crate::bt::DeviceKind::Other=>egui::include_image!("../assets/device-icons/bluetooth-linear.svg"),
    };
    let rect=c.rect(x,y,size,size);
    let egui::ImageSource::Bytes{uri,bytes}=icon else{return};
    let pixels=(rect.width()*c.ui.ctx().pixels_per_point()).ceil() as u32;
    let icon=egui::Image::from_bytes(format!("{uri}/{pixels}.svg"),bytes);
    if let Ok(egui::load::TexturePoll::Ready { texture })=icon.fit_to_exact_size(rect.size()).load_for_size(c.ui.ctx(),rect.size()) {
        c.painter.image(texture.id,rect,Rect::from_min_max(egui::Pos2::ZERO,egui::pos2(1.0,1.0)),Color32::WHITE);
    }
}

pub(crate) use crate::pet::PetState;

pub(crate) fn notice_badge(c:&Card,x:f32,y:f32,a:&Alert){
    if a.title=="蓝牙设备"{
        c.circle(x,y,11.0,if a.body.contains("已断开"){Color32::from_gray(95)}else{theme::BLUE});
        device_icon(c,a.bluetooth.map_or(crate::bt::DeviceKind::Other,|v|v.0),x-7.5,y-7.5,15.0);
    }else{notice_icon(&c.scaled(x,y,0.85),x,y,a);}
}
fn notice_icon(c:&Card,x:f32,y:f32,a:&Alert) {
    match a.title.as_str() {
        "系统音量"=>{
            c.poly(&[(x-9.0,y-3.0),(x-5.0,y-3.0),(x,y-7.0),(x,y+7.0),(x-5.0,y+3.0),(x-9.0,y+3.0)],theme::WHITE);
            c.arc(x,y,5.0,-0.8,0.8,1.5,theme::WHITE);c.arc(x,y,8.0,-0.7,0.7,1.3,theme::WHITE);
        },
        "大写锁定"=>{
            let ink=if a.body=="已开启"{theme::GREEN}else{theme::GRAY};
            c.poly(&[(x-8.0,y-1.0),(x,y-9.0),(x+8.0,y-1.0)],ink);
            c.fill(x-3.0,y-1.5,6.0,6.5,0.5,ink);
            c.fill(x-4.0,y+7.0,8.0,2.0,0.8,ink);
        },
        "键盘输入语言"=>{
            c.stroke(x-10.0,y-8.5,20.0,17.0,4.0,1.2,theme::GRAY);
            let label=match a.body.as_str(){"中文"=>"中","英语"=>"EN",_=>"Aa"};
            c.centered_text(x,y,label,if label=="EN"{9.0}else{11.0},W::Semibold,theme::WHITE,H::Center);
        },
        "网络连接"=>{
            let connected=a.body.contains("已连接");
            let ink=if connected{theme::GREEN}else{theme::GRAY};
            if a.body.starts_with("Wi-Fi") {
                for r in [4.0,8.0,12.0]{c.arc(x,y+6.0,r,-2.35,-0.79,1.7,ink);}
                c.circle(x,y+6.0,1.6,ink);
            }else if a.body.starts_with("以太网") {
                c.stroke(x-8.0,y-8.0,16.0,14.0,2.5,1.6,ink);
                for dx in [-4.0,-1.3,1.3,4.0]{c.seg((x+dx,y-5.0),(x+dx,y-1.0),1.2,ink);}
                c.path(&[(x-4.0,y+6.0),(x-4.0,y+9.0),(x+4.0,y+9.0),(x+4.0,y+6.0)],1.6,ink);
            }else{
                c.ring(x,y,8.0,1.4,ink);c.seg((x-8.0,y),(x+8.0,y),1.0,ink);
                c.stroke(x-3.5,y-8.0,7.0,16.0,3.5,1.0,ink);
            }
            if !connected{c.seg((x-8.0,y-8.0),(x+9.0,y+9.0),2.0,theme::WHITE);}
        },
        "电源已连接"|"已切换电池供电"|"电量偏低"=>{
            let low=a.title=="电量偏低";let ink=if low{theme::RED_TOP}else if a.title=="电源已连接"{theme::GREEN}else{theme::WHITE};
            c.stroke(x-11.0,y-6.0,20.0,12.0,3.0,1.5,ink);c.fill(x+10.0,y-2.5,2.0,5.0,0.8,ink);
            if low{c.fill(x-8.0,y-3.5,3.5,7.0,1.0,ink);}
            else if a.title=="电源已连接"{
                c.poly(&[(x,y-4.5),(x-4.0,y+0.8),(x+0.5,y+0.8)],ink);
                c.poly(&[(x+0.5,y-0.8),(x+4.0,y-0.8),(x-1.0,y+4.5)],ink);
            }else{c.fill(x-8.0,y-3.5,12.0,7.0,1.0,ink);}
        },
        "喝水时间到了"=>{
            c.poly(&[(x-7.0,y-1.0),(x+5.0,y-1.0),(x+3.5,y+8.0),(x-5.5,y+8.0)],theme::BLUE);
            c.path(&[(x-9.0,y-10.0),(x-6.5,y+9.0),(x+4.5,y+9.0),(x+7.0,y-10.0)],1.6,theme::CYAN);
            c.seg((x-9.0,y-10.0),(x+7.0,y-10.0),1.6,theme::CYAN);
            c.path(&[(x+6.5,y-6.0),(x+10.5,y-6.0),(x+11.5,y-3.0),(x+10.0,y+2.0),(x+5.5,y+3.0)],1.6,theme::CYAN);
            c.seg((x-5.0,y-6.5),(x-4.2,y-2.5),1.0,Color32::from_white_alpha(170));
        },
        "待办到时间了"=>crate::paint::checklist(c,x,y,23.0,Color32::from_rgb(190,180,255),Color32::from_rgb(43,35,66)),
        "会议提醒"=>{
            let label=a.meeting.as_ref().map(|(_,source)|source.as_str()).unwrap_or("");
            let source=if label.starts_with("邮件"){crate::calendar_ui::Source::Mail("")}
                else{crate::calendar_ui::Source::Calendar(crate::calendar::Provider::ALL.into_iter().find(|provider|provider.label()==label))};
            crate::calendar_ui::source_icon(c,source,x-14.0,y-14.0,28.0);
        },
        "专注计时"=>crate::paint::stopwatch(&c.scaled(x,y,0.65),x,y,8.0,0.0,theme::RED_TOP),
        _=>crate::paint::bell(&c.scaled(x,y,1.7),x,y,theme::GRAY,0.0),
    }
}

impl App {
    pub(crate) fn phase_secs(&self, phase: Phase) -> f32 {
        (match phase {
            Phase::Focus => self.settings.focus_minutes,
            Phase::Rest => self.settings.rest_minutes,
        }) as f32
            * 60.0
    }
    pub(crate) fn water_hours(&self, hour: u32) -> bool {
        let (a, b) = (self.settings.water_start, self.settings.water_end);
        if a == b {
            true
        } else if a < b {
            hour >= a && hour < b
        } else {
            hour >= a || hour < b
        }
    }
    pub(crate) fn pet_state(&self) -> PetState {
        if let Some((state, _)) = self.pet.reaction {
            state
        } else if self.pet.waking_until > self.t {
            PetState::Waking
        } else if self.pomo.running && self.pomo.phase == Phase::Focus {
            PetState::Focused
        } else if self.pet.worried {
            PetState::Worried
        } else if self.media.playing && self.media.is_music() {
            if self.notice.is_some() || self.hud_notice.is_some() {PetState::Awake}else{PetState::Music}
        } else if self.pet.hovered {
            PetState::Curious
        } else if self.idle_for() >= 180.0 {
            PetState::Sleeping
        } else if self.idle_for() >= self.opts.pet_idle {
            PetState::Dozing
        } else {
            PetState::Awake
        }
    }
    pub fn alert(
        &mut self,
        title: impl Into<String>,
        body: impl Into<String>,
        page: usize,
        system: bool,
    ) {
        let title = title.into();
        let transient = matches!(
            title.as_str(),
            "系统音量" | "大写锁定" | "键盘输入语言" | "网络连接" | "电源已连接" | "已切换电池供电" | "蓝牙设备"
        );
        let a = Alert {
            title,
            body: body.into(),
            page,
            at: self.t,
            duration: if transient { 2.5 } else { 8.0 },
            bluetooth: None,
            meeting: None,
        };
        self.enqueue_alert(a, system);
    }
    pub(crate) fn meeting_alert(&mut self, meeting: crate::mail::Meeting, source: String, page: usize) {
        self.enqueue_alert(Alert {
            title: "会议提醒".into(), body: format!("{} · {} · {}", meeting.title, meeting.at, source),
            page, at: self.t, duration: 12.0, bluetooth: None, meeting: Some((meeting, source)),
        }, true);
    }
    pub(crate) fn bluetooth_alert(&mut self, device: &crate::bt::Device, connected: bool) {
        if connected && matches!(device.kind,crate::bt::DeviceKind::Bone|crate::bt::DeviceKind::Earbuds|crate::bt::DeviceKind::Headphones|crate::bt::DeviceKind::Speaker) {
            self.alerts.retain(|a|a.title!="系统音量");
            if self.notice.as_ref().is_some_and(|a|a.title=="系统音量") {self.notice=None;}
        }
        let a = Alert {
            title: "蓝牙设备".into(),
            body: if connected { device.summary() } else { format!("{} · 已断开", device.name) },
            page: 5, at: self.t, duration: 2.5,
            bluetooth: Some((device.kind, connected)),
            meeting: None,
        };
        self.enqueue_alert(a, false);
    }
    fn enqueue_alert(&mut self, a: Alert, system: bool) {
        // 通知仅保存在本次运行的内存中。
        self.history.push_front(a.clone());
        self.history.truncate(50);
        if self.settings.notifications {
            if system {
                self.pending
                    .push(Cmd::Notify(a.title.clone(), a.body.clone()));
            }
            let same_event=|old:&Alert| match (&a.meeting,&old.meeting) {
                (Some((next,_)),Some((previous,_)))=>crate::calendar::dedupe_key(next)==crate::calendar::dedupe_key(previous),
                _=>old.title==a.title && (a.is_hud() || old.body==a.body),
            };
            if a.title != "蓝牙设备" { self.alerts.retain(|old| !same_event(old)); }
            if a.title != "蓝牙设备" && self.notice.as_ref().is_some_and(same_event) {
                self.notice = Some(a);
                return;
            }
            self.alerts.push_back(a);
            while self.alerts.len() > 8 {
                self.alerts.pop_front();
            }
        }
    }
    pub fn receive_notification(&mut self, app:String, id:String, body:String){
        if self.history.iter().any(|a|format!("{} · {}",a.title,a.body)==body){return;}
        let id=if id.is_empty(){app.clone()}else{id};
        if !self.notification_apps.observe(&id,&app) || !self.settings.notifications {return;}
        self.notification_bell_at=self.t;
        self.notification_bell_until=self.t+4.5;
        self.enqueue_alert(Alert{title:app,body,page:0,at:self.t,duration:5.0,bluetooth:None,meeting:None}, false);
    }
    pub(crate) fn tick_alerts(&mut self) {
        if self
            .notice
            .as_ref()
            .is_some_and(|a| self.t - a.at > a.duration)
        {
            self.notice = None;
        }
        if self.notice.is_none() {
            if let Some(mut a) = self.alerts.pop_front() {
                a.at = self.t;
                self.notice = Some(a);
            }
        }
    }
    pub(crate) fn draw_notice(&mut self, c:&Card,panel:Rect,a:&Alert,_cmds:&mut Vec<Cmd>){
        let nc=c.clipped(0.0,0.0,panel.width(),panel.height());
        if let Some((meeting,_))=&a.meeting {
            let secondary=Color32::from_gray(174);
            notice_icon(&nc.scaled(34.0,46.0,36.0/28.0),34.0,46.0,a);
            let now=chrono::Local::now();
            let start=chrono::DateTime::from_timestamp(meeting.start,0).unwrap().with_timezone(&chrono::Local);
            let left=meeting.start-now.timestamp();
            let countdown=if left>0 {format!("{} 分钟后",(left+59)/60)}else{"正在开始".into()};
            let countdown_width=nc.text_w(&countdown,10.5,W::Semibold)+16.0;
            let countdown_x=344.0-countdown_width;
            nc.centered_text(64.0,33.0,&start.format("%H:%M").to_string(),30.0,W::Semibold,theme::WHITE,H::Left);
            nc.fill(countdown_x,23.0,countdown_width,20.0,10.0,Color32::from_gray(28));
            nc.centered_text(countdown_x+countdown_width/2.0,33.0,&countdown,10.5,W::Semibold,theme::WHITE,H::Center);
            let overflow=(nc.text_w(&meeting.title,size::TITLE,W::Semibold)-280.0).max(0.0);
            let offset=(((self.t-a.at-1.2).max(0.0) as f32)*24.0).min(overflow);
            nc.clipped(64.0,52.0,280.0,24.0).centered_text(64.0-offset,64.0,&meeting.title,size::TITLE,W::Semibold,theme::WHITE,H::Left);
            nc.seg((16.0,84.0),(344.0,84.0),0.6,Color32::from_gray(36));
            let date=format!("{}{}",if start.date_naive()==now.date_naive(){"今天 · "}else{""},start.format("%-m月%-d日"));
            nc.centered_text(16.0,110.0,&date,11.0,W::Regular,secondary,H::Left);
            let (button,bc)=crate::paint::capsule(&nc,272.0,98.0,72.0,24.0,"meeting-action",Color32::from_gray(42),Color32::from_gray(56));
            bc.centered_text(308.0,110.0,"查看",10.5,W::Semibold,theme::WHITE,H::Center);
            if button.clicked() {
                self.notice=None;
                self.open_calendar(Some(meeting.clone()));
            }
            return;
        }
        if a.is_hud(){
            if a.title=="蓝牙设备" {
                let (name,status,battery)=bluetooth_notice_content(&a.body);
                nc.circle(26.0,24.0,14.0,if status=="已连接"{Color32::from_rgb(10,132,255)}else{Color32::from_gray(95)});
                let kind=a.bluetooth.map(|(kind,_)|kind).unwrap_or(crate::bt::DeviceKind::Other);
                device_icon(&nc,kind,17.0,15.0,18.0);
                let right=HUD_W-14.0;
                let name_end=if let Some(level)=battery {
                    let pct=format!("{level}%");
                    let pct_width=nc.text_w(&pct,10.5,W::Semibold);
                    let x=right-pct_width-22.0;
                    nc.stroke(x,20.0,15.0,8.0,2.0,1.0,Color32::from_gray(175));
                    nc.fill(x+15.5,22.5,2.0,3.0,0.8,Color32::from_gray(175));
                    let fill=11.0*level as f32/100.0;
                    if fill>0.0 {nc.fill(x+2.0,22.0,fill,4.0,1.0,if level<=20{theme::RED}else{theme::GREEN});}
                    nc.centered_text(right,24.0,&pct,10.5,W::Semibold,theme::WHITE,H::Right);
                    x-8.0
                } else {right};
                nc.centered_text(49.0,16.0,&nc.fit(name,11.0,W::Semibold,(name_end-49.0).max(20.0)),11.0,W::Semibold,theme::WHITE,H::Left);
                nc.centered_text(49.0,32.0,status,10.5,W::Regular,theme::GRAY,H::Left);
                return;
            }
            match a.title.as_str(){
                "系统音量"=>{
                    nc.poly(&[(17.0,16.0),(21.0,16.0),(26.0,12.0),(26.0,26.0),(21.0,22.0),(17.0,22.0)],theme::WHITE);
                    if self.volume_ui.is_some_and(|(_,muted)|muted){nc.seg((29.0,16.0),(34.0,22.0),1.4,theme::WHITE);nc.seg((34.0,16.0),(29.0,22.0),1.4,theme::WHITE);}
                    else{nc.arc(26.0,19.0,5.0,-0.8,0.8,1.5,theme::WHITE);nc.arc(26.0,19.0,8.0,-0.7,0.7,1.3,theme::WHITE);}
                },
                _=>notice_icon(&nc,24.0,19.0,a),
            }
            if a.title=="系统音量" {
                self.draw_volume_hud(&nc);
            } else {
                let (title,body)=if a.title=="键盘输入语言"{("输入语言",a.body.as_str())}
                    else if a.title=="已切换电池供电"{("电池供电",a.body.as_str())}
                    else if a.title=="网络连接"{a.body.split_once(" · ").unwrap_or(("网络",a.body.as_str()))}
                    else{(a.title.as_str(),a.body.as_str())};
                nc.centered_text(48.0,19.0,&nc.fit(title,11.0,W::Semibold,90.0),11.0,W::Semibold,theme::WHITE,H::Left);
                nc.centered_text(HUD_W-16.0,19.0,&nc.fit(body,10.5,W::Regular,92.0),10.5,W::Regular,if a.title=="大写锁定"&&a.body=="已开启"||a.title=="网络连接"&&a.body.contains("已连接"){theme::GREEN}else{theme::GRAY},H::Right);
            }
        } else {
            let is_water=a.page==2;let is_focus=a.page==1;let is_power=a.title=="电量偏低";
            notice_icon(&nc,22.0,22.0,a);
            let max=if is_water{125.0}else{195.0};
            nc.centered_text(43.0,14.0,&nc.fit(&a.title,11.0,W::Semibold,max),11.0,W::Semibold,theme::WHITE,H::Left);
            nc.centered_text(43.0,30.0,&nc.fit(&a.body,10.5,W::Regular,max),10.5,W::Regular,theme::GRAY,H::Left);
            if is_water {
                if notice_button(&nc,181.0,12.0,62.0,"稍后","water-snooze"){self.last_reminder=self.t-self.settings.water_minutes as f64*60.0+600.0;self.notice=None;}
                if notice_button(&nc,250.0,12.0,68.0,"喝一口","water-confirm"){self.notice=None;self.drink_water();}
            }else{
                let label=if is_focus{"开始"}else if is_power{"知道了"}else{"查看"};
                if notice_button(&nc,257.0,12.0,60.0,label,"notice-action"){
                    self.notice=None;if is_power{return;}if is_focus{self.pomo.running=true;}else if a.page<6{
                        self.show_page(a.page);
                        if a.page==4 {
                            if let Some(index)=self.todos.items.iter().position(|task|task.text==a.body && task.done_on.is_none()){
                                self.todo_first=index.min(self.todos.items.len().saturating_sub(3));
                            }
                        }
                    }else{self.settings_tab=6;self.open_settings();}
                }
            }
        }
    }
    /// v0.5.4：可交互音量条。样式参考 oyenza-studio（MIT）的 HUD 音量条：
    /// 圆角胶囊、填充缓动、拖动时加粗并出现手柄；拖动/滚轮改音量，点喇叭静音。
    pub(crate) fn draw_volume_hud(&mut self, nc: &Card) {
        let (v, muted) = self.volume_ui.unwrap_or((0, false));
        let v = v.min(100);
        // 喇叭：点击静音 / 恢复
        let icon = nc.hit(12.0, 9.0, 28.0, 22.0, "volume-mute");
        if icon.clicked() && nc.live {
            self.volume_ui = Some((v, !muted));
            self.volume_quiet_until = self.t + 0.9;
            self.pending.push(Cmd::ToggleMute);
        }
        // 滑条：拖动 / 点击定位；滚轮微调
        let track = nc.hit_drag(HUD_TRACK_X, 8.0, HUD_TRACK_W, 24.0, "volume-track");
        let mut want: Option<u32> = None;
        if track.dragged() || track.clicked() {
            if let Some(p) = track.interact_pointer_pos() {
                let x = nc.to_local(p).x;
                want = Some((((x - HUD_TRACK_X) / HUD_TRACK_W) * 100.0).clamp(0.0, 100.0).round() as u32);
            }
        } else if nc.live && track.hovered() {
            let dy = nc.ui.ctx().input(|i| i.raw_scroll_delta.y);
            if dy.abs() > 0.5 {
                want = Some(((v as f32 + (dy / 120.0) * 4.0).clamp(0.0, 100.0)).round() as u32);
            }
        }
        if let Some(nv) = want {
            if self.volume_ui.map(|(o, _)| o) != Some(nv) || muted {
                self.volume_ui = Some((nv, false));
                self.volume_quiet_until = self.t + 0.9;
                self.pending.push(Cmd::SetVolume(nv));
            }
        }
        let (v, muted) = self.volume_ui.unwrap_or((v, muted));
        let pressed = track.dragged() || track.is_pointer_button_down_on() || icon.is_pointer_button_down_on();
        let target = if muted { 0.0 } else { v as f32 / 100.0 };
        self.volume_a = crate::anim::approach(self.volume_a, target, if pressed { 34.0 } else { 13.0 }, self.dt);
        let h = if pressed { 7.0 } else { 5.0 };
        let y = 19.0 - h / 2.0;
        nc.fill(HUD_TRACK_X, y, HUD_TRACK_W, h, h / 2.0, Color32::from_white_alpha(if pressed { 46 } else { 34 }));
        if self.volume_a > 0.0 {
            let w = (HUD_TRACK_W * self.volume_a).max(h);
            let col = if muted { Color32::from_gray(150) } else { theme::WHITE };
            nc.fill(HUD_TRACK_X, y, w, h, h / 2.0, col);
            nc.fill(HUD_TRACK_X, y + 0.8, (w - 1.2).max(0.0), h * 0.4, h * 0.2, Color32::from_white_alpha(64));
            if pressed {
                nc.circle(HUD_TRACK_X + w - h / 2.0, y + h / 2.0, h * 0.95, col);
            }
        }
        let label = if muted { "静音".to_string() } else { format!("{v}%") };
        nc.centered_text(HUD_W - 26.0, 19.0, &label, 10.5, if pressed { W::Semibold } else { W::Regular }, if muted { theme::GRAY } else { theme::WHITE }, H::Center);
    }

    pub(crate) fn drink_water(&mut self) {        let hour = self.current_hour;
        let amount = self.settings.serving_ml;
        let was = self.water.percent();
        let before = self.water.drunk_ml;
        self.water.drink(amount, hour);
        let amount = self.water.drunk_ml.saturating_sub(before);
        self.last_drink = Some((amount, hour));
        self.last_reminder = self.t;
        self.pet.react(PetState::Satisfied);
        self.say(format!("已记录 {amount} ml"));
        if was < 100.0 && self.water.percent() >= 100.0 {
            self.alert("今日饮水目标达成", "做得不错，继续保持", 2, true);
        }
    }


}

/// 将本地提醒时间规范化，拒绝不存在的日期与超出范围的时分。
pub(crate) fn parse_due(value: &str, today: &str) -> Option<String> {
    let value = value.trim();
    let full = if value.len() == 5 {
        format!("{today} {value}")
    } else {
        value.to_string()
    };
    if !full.is_ascii() || full.len() != 16 {
        return None;
    }
    let b = full.as_bytes();
    if b[4] != b'-' || b[7] != b'-' || b[10] != b' ' || b[13] != b':' {
        return None;
    }
    let year = full[0..4].parse::<u32>().ok()?;
    let month = full[5..7].parse::<usize>().ok()?;
    let day = full[8..10].parse::<u32>().ok()?;
    let hour = full[11..13].parse::<u32>().ok()?;
    let minute = full[14..16].parse::<u32>().ok()?;
    if !(2000..=9999).contains(&year) || !(1..=12).contains(&month) || hour > 23 || minute > 59 {
        return None;
    }
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    if day == 0 || day > days[month - 1] {
        return None;
    }
    Some(full)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn number_stepper_clicks_follow_steps_and_limits() {
        for (initial,step,x,expected) in [(5u32,1,127.0,6),(5,1,13.0,4),(0,1,13.0,0),(10,1,127.0,10),(9,3,127.0,10),(1,3,13.0,0)] {
            let ctx=egui::Context::default();
            let mut value=initial;
            let pos=egui::pos2(x+20.0,34.0);
            let events=[vec![],vec![egui::Event::PointerMoved(pos)],vec![egui::Event::PointerButton{pos,button:egui::PointerButton::Primary,pressed:true,modifiers:Default::default()}],vec![egui::Event::PointerButton{pos,button:egui::PointerButton::Primary,pressed:false,modifiers:Default::default()}]];
            for events in events {
                let raw=egui::RawInput{screen_rect:Some(Rect::from_min_size(egui::Pos2::ZERO,egui::vec2(200.0,80.0))),events,..Default::default()};
                let _=ctx.run(raw,|ctx|{
                    egui::CentralPanel::default().frame(egui::Frame::new().inner_margin(20)).show(ctx,|ui|{
                        ui.horizontal(|ui|{number_stepper(ui,&mut value,0,10,"",step,76.0);});
                    });
                });
            }
            assert_eq!(value,expected,"初值 {initial}，步长 {step}，点击位置 {x}");
        }
    }

    #[test]
    fn number_stepper_fits_with_settings_button_padding() {
        let ctx=egui::Context::default();
        let raw=egui::RawInput{screen_rect:Some(Rect::from_min_size(egui::Pos2::ZERO,egui::vec2(340.0,240.0))),..Default::default()};
        let _=ctx.run(raw,|ctx|{
            egui::CentralPanel::default().frame(egui::Frame::new()).show(ctx,|ui|{
                ui.spacing_mut().button_padding=egui::vec2(10.0,6.0);
                for mut value in [0u32,25,180] {
                    let row=ui.horizontal(|ui|{number_stepper(ui,&mut value,0,180," 分钟",1,76.0);});
                    assert!(row.response.rect.width()<=140.1,"数值控件超出预留宽度：{}",row.response.rect.width());
                }
                for mut value in [25u32,5,15] {setting_number(ui,"计时时长",&mut value,1,180," 分钟",1.0);}
                assert!(ui.min_rect().right()<=340.1,"连续数值行不应撑宽卡片");
            });
        });
    }

    #[test]
    fn settings_groups_keep_every_page_reachable() {
        let groups = ["常规", "音乐与歌词", "专注与饮水", "专注与饮水", "待办与置物", "待办与置物", "通知、邮箱与日程", "通知、邮箱与日程", "通知、邮箱与日程", "关于", "页面管理", "AI 额度"];
        for (tab, expected) in groups.iter().enumerate() {
            let matches: Vec<_> = SETTINGS_GROUPS.iter().filter(|(_,start,end)| (*start..=*end).contains(&tab)).collect();
            assert_eq!(matches.len(), 1, "页面 {tab} 必须且只能属于一个分组");
            assert_eq!(matches[0].0, *expected);
        }
    }

    #[test]
    fn bluetooth_notice_uses_real_connection_and_battery_data(){
        assert_eq!(bluetooth_notice_content("OpenMove by Shokz · 电量 80%"),("OpenMove by Shokz","已连接",Some(80)));
        assert_eq!(bluetooth_notice_content("耳机"),("耳机","已连接",None));
        assert_eq!(bluetooth_notice_content("已断开连接"),("蓝牙设备","已断开",None));
        assert_eq!(bluetooth_notice_content("青龙键盘 · 已断开"),("青龙键盘","已断开",None));
        assert_eq!(bluetooth_notice_content("耳机 · 电量 101%"),("耳机 · 电量 101%","已连接",None));
    }

    #[test]
    fn device_icons_render_as_visible_svg() {
        let icons: &[&[u8]] = &[
            include_bytes!("../assets/device-icons/headphones-round-bold.svg"),
            include_bytes!("../assets/device-icons/speaker-bold.svg"),
            include_bytes!("../assets/device-icons/keyboard-bold.svg"),
            include_bytes!("../assets/device-icons/mouse-bold.svg"),
            include_bytes!("../assets/device-icons/gamepad-bold.svg"),
            include_bytes!("../assets/device-icons/smartphone-bold.svg"),
            include_bytes!("../assets/device-icons/bluetooth-linear.svg"),
        ];
        for bytes in icons {
            let image=egui_extras::image::load_svg_bytes(bytes).unwrap();
            assert!(image.pixels.iter().any(|p|p.a()>0));
        }
    }

    #[test]
    fn due_time_checks_calendar_and_format() {
        assert_eq!(
            parse_due("09:30", "2026-10-29").as_deref(),
            Some("2026-10-29 09:30")
        );
        assert!(parse_due("2026-02-29 12:00", "").is_none());
        assert!(parse_due("2028-02-29 12:00", "").is_some());
        for value in [
            "24:00",
            "12:60",
            "明天中午",
            "2026-13-01 10:00",
            "2026-00-01 10:00",
        ] {
            assert!(parse_due(value, "2026-10-29").is_none());
        }
    }
}

pub(crate) struct TaskDraft {index:usize,original:String,text:String,due:String,error:String,focused:bool,reminder_only:bool,pub(crate) picker:crate::task_picker::ReminderPicker}
impl App {
    pub(crate) fn open_task_dialog(&mut self,index:usize){
        if let Some(task)=self.todos.items.get(index){
            let draft=TaskDraft{index,original:task.text.clone(),text:task.text.clone(),due:task.due.clone().unwrap_or_default(),error:String::new(),focused:false,reminder_only:false,picker:crate::task_picker::ReminderPicker::new(task.due.as_deref())};
            self.show_page(4);self.task_dialog=Some(draft);
        }
    }
    pub(crate) fn open_task_reminder(&mut self,index:usize){
        self.open_task_dialog(index);
        if let Some(d)=self.task_dialog.as_mut(){d.reminder_only=true;d.picker.show();}
    }
    pub(crate) fn draw_task_editor(&mut self,c:&Card){
        if self.task_dialog.as_ref().is_some_and(|draft|draft.picker.open){self.draw_task_reminder(c);return;}
        let reminder_only=self.task_dialog.as_ref().is_some_and(|d|d.reminder_only);
        c.text(15.0,25.0,if reminder_only{"设置提醒"}else{"编辑待办"},size::TITLE,W::Bold,theme::WHITE,H::Left);
        c.text(43.0,48.0,if reminder_only{"待办"}else{"内容"},10.5,W::Regular,theme::GRAY,H::Left);
        c.fill(91.0,33.0,275.0,20.0,6.0,theme::CARD);
        if let Some(d)=self.task_dialog.as_ref().filter(|d|d.reminder_only){
            c.text(97.0,47.0,&c.fit(&d.text,11.5,W::Regular,260.0),11.5,W::Regular,theme::WHITE,H::Left);
        }
        c.text(43.0,75.0,"提醒",10.5,W::Regular,theme::GRAY,H::Left);
        let time=self.task_dialog.as_ref().map(|d|if d.due.is_empty(){"点击选择日期和时间".into()}else{d.due.clone()}).unwrap_or_default();
        if setting_button(c,91.0,60.0,275.0,&time,false,"task-select-reminder"){if let Some(d)=self.task_dialog.as_mut(){d.picker.show();}}
        if let Some(d)=&self.task_dialog {let hint=if d.error.is_empty(){"选择日期与时间，也可以暂不提醒"}else{&d.error};c.text(92.0,92.0,&c.fit(hint,10.5,W::Regular,270.0),10.5,W::Regular,if d.error.is_empty(){theme::GRAY}else{theme::RED},H::Left);}
        if reminder_only {
            let has_reminder=self.task_dialog.as_ref().and_then(|d|self.todos.items.get(d.index)).is_some_and(|task|task.due.is_some());
            if setting_button(c,188.0,101.0,88.0,if has_reminder{"取消提醒"}else{"暂不提醒"},false,"task-reminder-skip"){
                if let Some(d)=self.task_dialog.as_mut(){d.due.clear();}
                self.save_task_editor();
            }
        }else if setting_button(c,202.0,101.0,74.0,"取消",false,"task-cancel"){self.task_dialog=None;}
        if setting_button(c,286.0,101.0,80.0,if reminder_only{"选择时间"}else{"保存"},true,"task-save"){
            if reminder_only{if let Some(d)=self.task_dialog.as_mut(){d.picker.show();}}else{self.save_task_editor();}
        }
    }
    pub(crate) fn task_editor_inputs(&mut self,ui:&mut egui::Ui,origin:egui::Pos2,cmds:&mut Vec<Cmd>){
        let Some(d)=self.task_dialog.as_mut() else{return};
        if d.reminder_only || d.picker.open{return;}
        let name=ui.put(Rect::from_min_size(origin+egui::vec2(97.0,35.0),egui::vec2(260.0,17.0)),egui::TextEdit::singleline(&mut d.text).id(egui::Id::new("task-name-edit")).frame(false).font(theme::font(11.5,W::Regular)));
        let first_focus=!d.focused;
        if first_focus{name.request_focus();d.focused=true;cmds.push(Cmd::Focus);}
        // 新建内容与下一步共用这一帧，避免同一次回车又提交了空提醒。
        if !first_focus && ui.input(|i|i.key_pressed(egui::Key::Enter)){self.save_task_editor();}
    }
    fn draw_task_reminder(&mut self,c:&Card){
        use crate::task_picker::PickerAction;
        let Some(d)=self.task_dialog.as_mut().filter(|d|d.picker.open) else{return};
        let has_reminder=self.todos.items.get(d.index).is_some_and(|task|task.due.is_some());
        c.text(15.0,25.0,"待办",size::TITLE,W::Bold,theme::WHITE,H::Left);
        c.text(44.0,25.0,"清单",size::TITLE,W::Bold,theme::GREEN,H::Left);
        let done=self.todos.items.iter().filter(|task|task.done_on.is_some()).count();
        c.text(42.0,61.0,&format!("{done}/{}",self.todos.items.len()),29.0,W::Bold,theme::WHITE,H::Left);
        let action=d.picker.draw(c,&d.text,has_reminder);
        if std::mem::take(&mut d.picker.focus){self.pending.push(Cmd::Focus);}
        match action{
            Some(PickerAction::Save)=>{d.due=d.picker.due();d.picker.open=false;if d.reminder_only{self.save_task_editor();}},
            Some(PickerAction::Skip)=>{d.due.clear();d.picker.open=false;if d.reminder_only{self.save_task_editor();}},
            Some(PickerAction::Cancel)=>{d.picker.open=false;if d.reminder_only{self.task_dialog=None;}},
            None=>{},
        }
    }
    fn save_task_editor(&mut self){
        let Some(mut d)=self.task_dialog.take() else{return};
        let due=if d.due.trim().is_empty(){Some(None)}else{parse_due(&d.due,&self.today).map(Some)};
        if d.text.trim().is_empty(){d.error="请填写任务内容".into();}
        else if due.is_none(){d.error="时间格式应为 18:30 或完整日期时间".into();}
        else if self.todos.items.get(d.index).is_none_or(|t|t.text!=d.original){d.error="任务已变化，请取消后重新打开".into();}
        else{let item=&mut self.todos.items[d.index];item.text=d.text.trim().into();item.due=due.unwrap();item.reminded=false;self.todos.save();self.say("待办已保存");return;}
        self.task_dialog=Some(d);
    }
}

fn notice_button(c:&Card,x:f32,y:f32,w:f32,text:&str,id:&str)->bool {
    let (r,b)=crate::paint::capsule(c,x,y,w,20.0,id,Color32::from_gray(35),theme::BTN_HOVER);
    b.centered_text(x+w/2.0,y+10.0,text,10.5,W::Semibold,theme::WHITE,H::Center);
    r.clicked()
}
fn setting_button(c:&Card,x:f32,y:f32,w:f32,text:&str,on:bool,id:&str)->bool {
    let (r,b)=crate::paint::capsule(c,x,y,w,19.0,id,if on{Color32::from_rgb(39,94,150)}else{theme::BTN},theme::BTN_HOVER);
    b.text(x+w/2.0,y+13.3,text,10.5,W::Semibold,theme::WHITE,H::Center);
    r.clicked()
}
impl App {
    pub(crate) fn page_settings(&mut self,c:&Card) {
        let help=match self.settings_category{0=>"外观与系统",1=>"调整饮水量与提醒时间",2=>"调整每个阶段的时长",_=>"通知"};
        c.text(44.0,44.0,help,10.5,W::Regular,theme::GRAY,H::Left);
        let before=format!("{:?}",self.settings);
        match self.settings_category {
            0=>{
                let choices=[("吉祥物",self.settings.pet),("极简刘海",self.settings.minimal),("系统提示",self.settings.system_hud),("全屏隐藏",self.settings.hide_fullscreen)];
                for (i,(name,on)) in choices.iter().enumerate(){let x=42.0+(i%2) as f32*169.0;let y=57.0+(i/2) as f32*27.0;let text=format!("{}  {}",name,if *on{"开"}else{"关"});if setting_button(c,x,y,158.0,&text,*on,&format!("general-{i}")){match i{0=>self.set_pet(!self.settings.pet),1=>self.set_minimal(!self.settings.minimal),2=>self.settings.system_hud=!self.settings.system_hud,_=>self.settings.hide_fullscreen=!self.settings.hide_fullscreen}}}
                c.text(42.0,116.0,"岛仔收起时常驻；极简模式保留细线",10.5,W::Regular,theme::GRAY,H::Left);
            }
            1=>{
                let rows=[("每次",format!("{} ml",self.settings.serving_ml)),("间隔",format!("{} 分",self.settings.water_minutes)),("目标",format!("{:.1} L",self.water.goal_ml as f32/1000.0))];
                for (i,(label,value)) in rows.iter().enumerate(){let y=53.0+i as f32*23.0;c.text(44.0,y+13.0,label,10.5,W::Regular,theme::GRAY,H::Left);c.text(148.0,y+13.0,value,10.5,W::Semibold,theme::WHITE,H::Center);
                    if setting_button(c,90.0,y,25.0,"−",false,&format!("water-minus-{i}")){match i{0=>self.settings.serving_ml=self.settings.serving_ml.saturating_sub(50).max(50),1=>self.settings.water_minutes=self.settings.water_minutes.saturating_sub(5).max(5),_=>{self.water.goal_ml=self.water.goal_ml.saturating_sub(250).max(500);self.water.save();}}}
                    if setting_button(c,180.0,y,25.0,"+",false,&format!("water-plus-{i}")){match i{0=>self.settings.serving_ml=(self.settings.serving_ml+50).min(1000),1=>self.settings.water_minutes=(self.settings.water_minutes+5).min(180),_=>{self.water.goal_ml=(self.water.goal_ml+250).min(6000);self.water.save();}}}}
                c.text(223.0,66.0,"提醒时段",10.5,W::Semibold,theme::WHITE,H::Left);
                for i in 0..2 {let y=73.0+i as f32*23.0;let hour=if i==0{self.settings.water_start}else{self.settings.water_end};c.text(250.0,y+13.0,&format!("{:02}:00",hour),10.5,W::Bold,theme::WHITE,H::Center);
                    if setting_button(c,287.0,y,25.0,"−",false,&format!("hour-minus-{i}")){if i==0{self.settings.water_start=(hour+23)%24}else{self.settings.water_end=(hour+23)%24}}
                    if setting_button(c,318.0,y,25.0,"+",false,&format!("hour-plus-{i}")){if i==0{self.settings.water_start=(hour+1)%24}else{self.settings.water_end=(hour+1)%24}}}
            }
            2=>{
                let values=[self.settings.focus_minutes,self.settings.rest_minutes];
                for (i,name) in ["专注","休息"].iter().enumerate(){let y=53.0+i as f32*23.0;c.text(48.0,y+13.0,name,10.5,W::Regular,theme::GRAY,H::Left);c.text(206.0,y+13.0,&format!("{} 分钟",values[i]),11.0,W::Bold,theme::WHITE,H::Center);
                    if setting_button(c,131.0,y,27.0,"−",false,&format!("focus-minus-{i}")){if i==0{self.settings.focus_minutes=values[i].saturating_sub(1).max(1)}else{self.settings.rest_minutes=values[i].saturating_sub(1).max(1)}}
                    if setting_button(c,255.0,y,27.0,"+",false,&format!("focus-plus-{i}")){if i==0{self.settings.focus_minutes=(values[i]+1).min(180)}else{self.settings.rest_minutes=(values[i]+1).min(60)}}}
                c.text(315.0,93.0,"下次生效",10.5,W::Regular,theme::GRAY,H::Center);
            }
            _=>{
                if setting_button(c,43.0,56.0,152.0,if self.settings.notifications{"提醒 · 开"}else{"提醒 · 关"},self.settings.notifications,"set-notice"){self.settings.notifications=!self.settings.notifications;}
                if setting_button(c,212.0,56.0,153.0,if self.settings.sound{"声音 · 开"}else{"声音 · 关"},self.settings.sound,"set-sound"){self.settings.sound=!self.settings.sound;}
                if setting_button(c,43.0,84.0,152.0,if self.settings.read_notifications{"断开系统通知"}else{"连接系统通知"},self.settings.read_notifications,"set-listener"){self.pending.push(if self.settings.read_notifications{Cmd::DisconnectNotifications}else{Cmd::RequestNotifications});}
                let status=c.fit(&self.listener_status,10.5,W::Regular,170.0);c.text(210.0,97.0,&status,10.5,W::Regular,theme::GRAY,H::Left);
            }
        }
        if before!=format!("{:?}",self.settings){self.settings.save();self.last_reminder=self.t;}
    }
}

fn switch(ui:&mut egui::Ui,value:&mut bool)->bool {
    let (rect,response)=ui.allocate_exact_size(egui::vec2(34.0,20.0),egui::Sense::click());
    if response.clicked(){*value=!*value;}
    let fill=if *value{Color32::from_rgb(48,184,92)}else{Color32::from_rgb(82,82,87)};
    ui.painter().rect_filled(rect,10.0,fill);
    let x=if *value{rect.right()-10.0}else{rect.left()+10.0};
    ui.painter().circle_filled(egui::pos2(x,rect.center().y+0.5),7.5,Color32::from_black_alpha(35));
    ui.painter().circle_filled(egui::pos2(x,rect.center().y),7.0,Color32::WHITE);
    response.clicked()
}
pub(crate) fn setting_row(ui:&mut egui::Ui,title:&str,description:&str,value:&mut bool){
    ui.horizontal(|ui|{
        ui.vertical(|ui|{
            ui.set_min_width((ui.available_width()-42.0).max(100.0));
            ui.label(egui::RichText::new(title).size(13.0).color(Color32::WHITE));
            if !description.is_empty(){ui.label(egui::RichText::new(description).size(11.5).color(Color32::from_gray(195)));}
        });
        switch(ui,value);
    });
}
pub(crate) fn settings_card(ui:&mut egui::Ui,add:impl FnOnce(&mut egui::Ui)) {
    egui::Frame::new().fill(Color32::from_rgb(35,35,38))
        .corner_radius(10.0).inner_margin(egui::Margin::symmetric(14,10))
        .show(ui,|ui|{ui.set_min_width(ui.available_width());add(ui);});
}
pub(crate) fn settings_label(ui:&mut egui::Ui,text:&str){
    ui.add_space(9.0);
    ui.label(egui::RichText::new(text).size(11.5).color(Color32::from_gray(185)));
    ui.add_space(2.0);
}
fn setting_number<N:egui::emath::Numeric>(ui:&mut egui::Ui,title:&str,value:&mut N,min:N,max:N,suffix:&str,speed:f64)->bool{
    let mut changed=false;
    ui.horizontal(|ui|{
        ui.label(egui::RichText::new(title).size(12.0).color(Color32::WHITE));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|{
            ui.allocate_ui_with_layout(egui::vec2(140.0,30.0),egui::Layout::left_to_right(egui::Align::Center),|ui|{
                changed=number_stepper(ui,value,min,max,suffix,N::from_f64(speed),76.0);
            });
        });
    });
    changed
}
fn number_stepper<N:egui::emath::Numeric>(ui:&mut egui::Ui,value:&mut N,min:N,max:N,suffix:&str,step:N,field_width:f32)->bool{
    let mut changed=false;
    ui.scope(|ui|{
        ui.spacing_mut().item_spacing.x=6.0;
        ui.spacing_mut().button_padding=egui::vec2(4.0,2.0);
        let button=|text|egui::Button::new(egui::RichText::new(text).size(16.0).color(Color32::WHITE))
            .fill(Color32::from_rgb(48,48,51)).stroke(egui::Stroke::NONE).corner_radius(13.0);
        let decrease=ui.add_enabled(*value>min,button("−").min_size(egui::vec2(26.0,26.0))).on_hover_text("减少");
        if decrease.clicked(){
            *value=N::from_f64((value.to_f64()-step.to_f64()).max(min.to_f64()));changed=true;
        }
        let field=ui.add_sized([field_width,28.0],egui::DragValue::new(value).range(min..=max).speed(step.to_f64()).suffix(suffix).fixed_decimals(if N::INTEGRAL{0}else{2}));
        changed|=field.changed();
        if ui.add_enabled(*value<max,button("+").min_size(egui::vec2(26.0,26.0))).on_hover_text("增加").clicked(){
            *value=N::from_f64((value.to_f64()+step.to_f64()).min(max.to_f64()));changed=true;
        }
    });
    changed
}
fn setting_choices(ui:&mut egui::Ui,title:&str,value:&mut u32,values:[u32;3],unit:&str){
    ui.horizontal(|ui|{
        ui.label(egui::RichText::new(title).size(13.0).color(Color32::WHITE));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|{
            ui.allocate_ui_with_layout(egui::vec2(192.0,29.0),egui::Layout::left_to_right(egui::Align::Center),|ui|{
                egui::Frame::new().fill(Color32::from_rgb(28,28,30)).corner_radius(7.0).inner_margin(2).show(ui,|ui|{
                    ui.spacing_mut().item_spacing.x=1.0;
                    ui.horizontal(|ui|{
                        for option in values{
                            let selected=*value==option;
                            let button=egui::Button::new(egui::RichText::new(format!("{option} {unit}")).size(11.5).color(if selected{Color32::WHITE}else{Color32::from_gray(205)}))
                                .fill(if selected{Color32::from_rgb(72,72,76)}else{Color32::TRANSPARENT})
                                .stroke(egui::Stroke::NONE).corner_radius(5.0);
                            if ui.add_sized([62.0,25.0],button).clicked(){*value=option;}
                        }
                    });
                });
            });
        });
    });
}
const SETTINGS_GROUPS: [(&str,usize,usize);8] = [
    ("常规",0,0), ("页面管理",10,10), ("音乐与歌词",1,1), ("专注与饮水",2,3),
    ("待办与置物",4,5), ("通知、邮箱与日程",6,8), ("AI 额度",11,11), ("关于",9,9),
];

impl App {
    pub fn settings_ui(&mut self,ctx:&egui::Context,autostart:bool){
        self.updates.poll(ctx);
        self.updates.dialog_ui(ctx);
        if !self.settings_open{return;}
        let id=egui::ViewportId::from_hash_of("daozai-settings");
        let focus_requested=std::mem::take(&mut self.settings_focus_requested);
        ctx.show_viewport_immediate(id,egui::ViewportBuilder::default().with_title("岛仔设置").with_icon(theme::app_icon()).with_inner_size([690.0,560.0]).with_min_inner_size([600.0,450.0]).with_active(true).with_window_level(egui::WindowLevel::Normal),|ctx,_|{
            if focus_requested {
                ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            }
            if ctx.input(|i|i.viewport().close_requested()){self.settings_open=false;}
            let before=format!("{:?}",self.settings);
            egui::SidePanel::left("settings-sidebar").resizable(false).exact_width(180.0).frame(egui::Frame::new().fill(Color32::from_rgb(29,29,32)).stroke(egui::Stroke::new(1.0,Color32::from_rgb(42,42,45))).inner_margin(egui::Margin::symmetric(10,12))).show(ctx,|ui|{
                ui.add_space(5.0);
                ui.label(egui::RichText::new("设置").size(11.0).color(Color32::from_gray(150)));
                ui.add_space(7.0);
                for (name,start,end) in SETTINGS_GROUPS {
                    if start==9 {ui.add_space((ui.available_height()-38.0).max(0.0));}
                    let selected=(start..=end).contains(&self.settings_tab);
                    let (rect,response)=ui.allocate_exact_size(egui::vec2(ui.available_width(),36.0),egui::Sense::click());
                    if response.clicked() && !selected{self.settings_tab=start;}
                    if selected || response.hovered(){
                        ui.painter().rect_filled(rect,7.0,if selected{Color32::from_rgb(43,43,46)}else{Color32::from_rgb(30,30,33)});
                    }
                    let icon=match start {
                        0=>egui::include_image!("../assets/settings-icons/system/general.svg"),
                        11=>egui::include_image!("../assets/settings-icons/system/quota.svg"),
                        10=>egui::include_image!("../assets/settings-icons/system/pages.svg"),
                        1=>egui::include_image!("../assets/settings-icons/system/music.svg"),
                        2=>egui::include_image!("../assets/settings-icons/system/focus.svg"),
                        4=>egui::include_image!("../assets/settings-icons/system/tasks.svg"),
                        6=>egui::include_image!("../assets/settings-icons/system/notifications.svg"),
                        _=>egui::include_image!("../assets/settings-icons/system/about.svg"),
                    };
                    let icon_bg=egui::Rect::from_center_size(egui::pos2(rect.left()+21.0,rect.center().y),egui::vec2(25.0,25.0));
                    egui::Image::new(icon).paint_at(ui,icon_bg);
                    ui.painter().text(egui::pos2(rect.left()+40.0,rect.center().y),egui::Align2::LEFT_CENTER,name,egui::FontId::proportional(13.0),if selected{Color32::WHITE}else{Color32::from_gray(225)});
                    ui.add_space(3.0);
                }
            });
            egui::CentralPanel::default().frame(egui::Frame::new().fill(Color32::from_rgb(25,25,27)).inner_margin(egui::Margin::symmetric(22,16))).show(ctx,|ui|{
                ui.spacing_mut().item_spacing=egui::vec2(8.0,5.0);
                ui.spacing_mut().button_padding=egui::vec2(10.0,6.0);
                ui.style_mut().text_styles.insert(egui::TextStyle::Small,egui::FontId::proportional(12.0));
                ui.visuals_mut().override_text_color=Some(Color32::WHITE);
                ui.visuals_mut().extreme_bg_color=Color32::from_rgb(25,25,28);
                ui.visuals_mut().widgets.inactive.bg_fill=Color32::from_rgb(50,50,54);
                ui.visuals_mut().widgets.hovered.bg_fill=Color32::from_rgb(65,65,71);
                ui.visuals_mut().widgets.inactive.corner_radius=egui::CornerRadius::same(6);
                ui.visuals_mut().widgets.hovered.corner_radius=egui::CornerRadius::same(6);
                ui.visuals_mut().widgets.active.corner_radius=egui::CornerRadius::same(6);
                ui.visuals_mut().selection.bg_fill=Color32::from_rgb(45,105,165);
                let titles=["常规","音乐与歌词","专注","饮水","待办","置物","通知","邮箱","日程","关于","页面管理","AI 额度"];
                ui.label(egui::RichText::new(titles[self.settings_tab.min(11)]).size(18.0).strong().color(Color32::WHITE));
                if let Some((_,start,end))=SETTINGS_GROUPS.iter().find(|(_,start,end)|(*start..=*end).contains(&self.settings_tab)) {
                    if start!=end {
                        ui.add_space(7.0);
                        egui::Frame::new().fill(Color32::from_rgb(33,33,36)).corner_radius(8.0).inner_margin(3).show(ui,|ui|{
                            ui.spacing_mut().item_spacing.x=2.0;
                            ui.horizontal(|ui|{
                                for index in *start..=*end {
                                    let selected=self.settings_tab==index;
                                    let button=egui::Button::new(egui::RichText::new(titles[index]).size(12.0).color(if selected{Color32::WHITE}else{Color32::from_gray(210)}))
                                        .fill(if selected{Color32::from_rgb(70,70,74)}else{Color32::TRANSPARENT})
                                        .stroke(egui::Stroke::NONE).corner_radius(6.0);
                                    if ui.add_sized([68.0,26.0],button).clicked(){self.settings_tab=index;}
                                }
                            });
                        });
                    }
                }
                ui.add_space(6.0);
                egui::ScrollArea::vertical().id_salt(("settings-scroll",self.settings_tab)).auto_shrink([false,false]).show(ui,|ui|{
                    match self.settings_tab {
                    11=>self.quota_settings(ui),
                    10=>{
                        ui.label(egui::RichText::new("选择需要的页面和状态组件。").size(12.0).color(Color32::from_gray(180)));
                        settings_label(ui,"常驻排列");
                        settings_card(ui,|ui|self.resident_order_settings(ui));
                        settings_label(ui,"状态组件");
                        settings_card(ui,|ui|{
                            setting_row(ui,"电量","本机电量、充电状态与提醒；无电池设备自动隐藏",&mut self.settings.show_battery);
                            ui.separator();
                            setting_row(ui,"岛上歌词","收起时显示正在唱的那一句",&mut self.settings.island_lyrics);
                        });
                        settings_label(ui,"功能页面");
                        settings_card(ui,|ui|{
                            let descriptions=["播放控制与同步歌词","番茄钟与休息计时","饮水记录与提醒","临时收纳文件位置","任务清单与到期提醒","邮件摘要与验证码","Agent 剩余额度和账号切换","月历、邮件邀请与工作日程"];
                            ui.label(egui::RichText::new("拖动左侧手柄调整顺序，松开后自动应用到岛。").size(11.5).color(Color32::from_gray(180)));
                            #[derive(Clone)]
                            struct PageDrag(usize);
                            let mut movement=None;
                            for (index,page) in self.settings.page_order.into_iter().enumerate() {
                                if index>0 {ui.separator();}
                                let mut visible=self.settings.visible_pages[page];
                                let can_change=!visible || self.settings.visible_pages.iter().filter(|enabled|**enabled).count()>1;
                                let row=ui.push_id(("page-order",page),|ui|ui.horizontal(|ui|{
                                    let (rect,handle)=ui.allocate_exact_size(egui::vec2(20.0,40.0),egui::Sense::drag());
                                    for x in [-3.0,3.0] {for y in [-6.0,0.0,6.0] {
                                        ui.painter().circle_filled(rect.center()+egui::vec2(x,y),1.4,Color32::from_gray(155));
                                    }}
                                    handle.on_hover_cursor(egui::CursorIcon::Grab).dnd_set_drag_payload(PageDrag(page));
                                    ui.add_enabled_ui(can_change,|ui|{setting_row(ui,PAGE_NAMES[page],descriptions[page],&mut visible);});
                                })).inner.response;
                                if let Some(payload)=row.dnd_hover_payload::<PageDrag>() {
                                    if payload.0!=page {
                                        ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
                                        let y=if self.settings.page_order.iter().position(|p|*p==payload.0).unwrap()<index {row.rect.bottom()}else{row.rect.top()};
                                        ui.painter().line_segment([egui::pos2(row.rect.left(),y),egui::pos2(row.rect.right(),y)],egui::Stroke::new(2.0,Color32::from_rgb(70,150,255)));
                                    }
                                }
                                if let Some(payload)=row.dnd_release_payload::<PageDrag>() {movement=Some((payload.0,index));}
                                if visible!=self.settings.visible_pages[page] {self.set_page_visible(page,visible);}
                            }
                            if let Some((page,to))=movement {
                                let from=self.settings.page_order.iter().position(|p|*p==page).unwrap();
                                if from<to {self.settings.page_order[from..=to].rotate_left(1);}
                                else {self.settings.page_order[to..=from].rotate_right(1);}
                            }
                            if self.settings.visible_pages.iter().filter(|enabled|**enabled).count()==1 {
                                ui.add_space(5.0);
                                ui.label(egui::RichText::new("至少保留一个页面。关闭仅隐藏页面，数据和提醒设置保留。").size(11.0).color(Color32::from_gray(180)));
                            }
                        });
                    }
                    0=>{
                        settings_label(ui,"启动");
                        settings_card(ui,|ui|{
                            let mut a=autostart;
                            setting_row(ui,"开机启动","登录 Windows 时自动打开岛仔",&mut a);
                            if a!=autostart{self.pending.push(Cmd::ToggleAutostart);}
                        });
                        settings_label(ui,"岛体显示");
                        settings_card(ui,|ui|{
                            let mut minimal=self.settings.minimal;
                            setting_row(ui,"极简刘海","收起时显示一条细线",&mut minimal);
                            if minimal!=self.settings.minimal {self.set_minimal(minimal);}
                            ui.separator();
                            setting_row(ui,"浏览器自动极简","浏览器贴顶时收成细线，移开或切走后恢复",&mut self.settings.browser_minimal);
                            ui.separator();
                            let mut pet=self.settings.pet;
                            setting_row(ui,"岛仔吉祥物","收起时常驻，播放音乐时一起唱歌",&mut pet);
                            if pet!=self.settings.pet {self.set_pet(pet);}
                        });
                        settings_label(ui,"声音与显示");
                        settings_card(ui,|ui|{
                            setting_row(ui,"提醒声音","喝水、待办和专注提醒",&mut self.settings.sound);
                            ui.separator();
                            setting_row(ui,"全屏时隐藏","游戏、视频和演示全屏时隐藏",&mut self.settings.hide_fullscreen);
                        });
                    }
                    1=>{
                        settings_label(ui,"歌词");
                        settings_card(ui,|ui|{
                            setting_row(ui,"展开时显示歌词","在播放器右侧同步滚动显示歌词",&mut self.settings.lyrics_enabled);
                            self.show_lyrics=self.settings.lyrics_enabled;
                            ui.separator();
                            setting_row(ui,"岛上歌词","收起时显示当前句；长句分段切换，无歌词时保留布局",&mut self.settings.island_lyrics);
                        });
                        settings_label(ui,"歌词显示时间");
                        settings_card(ui,|ui|{
                            let mut advance=-self.settings.lyric_delay_ms as f64/1000.0;
                            if setting_number(ui,"时间偏移",&mut advance,-5.0,5.0," 秒",0.1){self.settings.lyric_delay_ms=(-advance*1000.0).round()as i32;}
                            ui.add_space(3.0);
                            ui.small("正数提前，负数延后；默认提前 0.35 秒。");
                        });
                    }
                    2=>{
                        settings_label(ui,"计时时长");
                        settings_card(ui,|ui|{
                            setting_number(ui,"专注",&mut self.settings.focus_minutes,1,180," 分钟",1.0);
                            ui.separator();
                            setting_number(ui,"休息",&mut self.settings.rest_minutes,1,60," 分钟",1.0);
                            ui.add_space(3.0);
                            ui.small("调整后从下一阶段开始生效。");
                        });
                    }
                    3=>{
                        settings_label(ui,"喝水提醒");
                        settings_card(ui,|ui|{
                            let old_remind=self.water.remind;
                            setting_row(ui,"定时提醒","达到每日目标后停止提醒",&mut self.water.remind);
                            if self.water.remind!=old_remind{self.water.save();self.last_reminder=self.t;}
                            ui.separator();
                            if setting_number(ui,"每日目标",&mut self.water.goal_ml,500,6000," ml",50.0){self.water.save();}
                            setting_number(ui,"每次记录",&mut self.settings.serving_ml,50,1000," ml",25.0);
                            setting_number(ui,"提醒间隔",&mut self.settings.water_minutes,5,180," 分钟",1.0);
                            ui.horizontal(|ui|{
                                ui.label(egui::RichText::new("提醒时段").size(12.0).color(Color32::WHITE));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|{
                                    ui.allocate_ui_with_layout(egui::vec2(252.0,30.0),egui::Layout::left_to_right(egui::Align::Center),|ui|{
                                        number_stepper(ui,&mut self.settings.water_start,0,23," 时",1,48.0);
                                        ui.label("至");
                                        number_stepper(ui,&mut self.settings.water_end,0,24," 时",1,48.0);
                                    });
                                });
                            });
                        });
                    }
                    4=>{
                        settings_label(ui,"待办清单");
                        settings_card(ui,|ui|{
                            ui.label(egui::RichText::new("在待办页添加、编辑任务并设置提醒时间。").size(13.0));
                            ui.add_space(5.0);
                            if ui.add_sized([120.0,32.0],egui::Button::new("打开待办页")).clicked(){self.settings_open=false;self.show_page(4);}
                        });
                    }
                    5=>{
                        settings_label(ui,"置物架");
                        settings_card(ui,|ui|{
                            ui.label(egui::RichText::new("保存文件位置，移除条目不会删除原文件。").size(13.0));
                            ui.label(egui::RichText::new(format!("当前 {} 项",self.shelf.items.len())).size(12.0).color(Color32::from_gray(180)));
                            ui.add_space(5.0);
                            if ui.add_sized([120.0,32.0],egui::Button::new("打开置物架")).clicked(){self.settings_open=false;self.show_page(3);}
                        });
                    }
                    6=>{
                        settings_label(ui,"系统提示");
                        settings_card(ui,|ui|{
                            setting_row(ui,"系统状态","音量、充电、网络和键盘状态",&mut self.settings.system_hud);
                            ui.separator();
                            setting_row(ui,"岛仔提醒","在岛上显示饮水、专注和待办提醒",&mut self.settings.notifications);
                        });
                        settings_label(ui,"应用通知");
                        settings_card(ui,|ui|{
                            ui.label(egui::RichText::new(&self.listener_status).size(13.0));
                            if ui.add_sized([150.0,32.0],egui::Button::new(if self.settings.read_notifications{"断开系统通知"}else{"连接 Windows 通知"})).clicked(){self.pending.push(if self.settings.read_notifications{Cmd::DisconnectNotifications}else{Cmd::RequestNotifications});}
                            ui.label(egui::RichText::new("新通知到达时，刘海右侧会短暂显示铃铛。").size(12.0).color(Color32::from_gray(180)));
                            ui.separator();
                            ui.label(egui::RichText::new("收到通知的应用会出现在这里，可分别开启或关闭。").size(12.0).color(Color32::from_gray(180)));
                            let before_apps=self.notification_apps.items.clone();
                            for (_,name,enabled) in &mut self.notification_apps.items {setting_row(ui,name,"",enabled);}
                            if before_apps!=self.notification_apps.items{self.notification_apps.save();}
                        });
                    }
                    7=>{
                        settings_label(ui,"邮件提醒");
                        settings_card(ui,|ui|{
                            setting_row(ui,"新邮件提醒","新邮件到达时显示提醒；岛上歌词模式显示右侧图标",&mut self.settings.mail_enabled);
                            ui.separator();
                            setting_row(ui,"验证码高亮","识别到验证码时在岛上放大显示，点击即复制",&mut self.settings.mail_code);
                            ui.separator();
                            setting_choices(ui,"检查间隔",&mut self.settings.mail_interval,[60,120,300],"秒");
                            ui.label(egui::RichText::new("支持推送时实时收信，并每 10 秒补查；以上间隔仅用于不支持推送的邮箱。").size(11.5).color(Color32::from_gray(195)));
                            ui.separator();
                            setting_choices(ui,"会议提前",&mut self.settings.mail_ics,[5,15,30],"分钟");
                            ui.label(egui::RichText::new("根据邮件中的日历邀请提醒，与日程设置共用。").size(11.5).color(Color32::from_gray(195)));
                        });
                        settings_label(ui,"已添加的邮箱");
                        settings_card(ui,|ui|{
                            if self.mail.accounts.is_empty(){ui.label(egui::RichText::new("还没有邮箱。在下面添加，保存后自动连接。").size(12.0).color(Color32::from_gray(195)));}
                            let total=self.mail.accounts.len();
                            let(mut changed,mut remove,mut test)=(false,None,None);
                            for i in 0..total{
                                let(email,provider)=(self.mail.accounts[i].email.clone(),self.mail.accounts[i].provider);
                                let status=self.mail.status_of(&email).label();
                                let mut on=self.mail.accounts[i].enabled;
                                if i>0{ui.separator();}
                                ui.label(egui::RichText::new(&email).size(13.0).color(Color32::WHITE));
                                ui.label(egui::RichText::new(format!("{} · {}",provider.label(),status)).size(12.0).color(Color32::from_gray(180)));
                                ui.horizontal(|ui|{
                                    if ui.add_sized([66.0,30.0],egui::Button::new("测试")).clicked(){test=Some(i);}
                                    if ui.add_sized([66.0,30.0],egui::Button::new("删除")).clicked(){remove=Some(i);}
                                    ui.add_space(8.0);
                                    ui.label(egui::RichText::new("启用").size(12.0).color(Color32::from_gray(190)));
                                    if switch(ui,&mut on){self.mail.accounts[i].enabled=on;changed=true;}
                                });
                            }
                            if changed{self.mail_accounts_changed();}
                            if let Some(i)=remove{
                                let email=self.mail.accounts[i].email.clone();
                                self.mail.accounts.remove(i);
                                crate::mail::secrets::delete(&email);
                                self.mail.status.remove(&email.to_ascii_lowercase());
                                self.mail.note=format!("{email}：已移除，凭据已从系统凭据管理器删除");
                                self.mail_accounts_changed();
                            }
                            if let Some(i)=test{
                                let account=self.mail.accounts[i].clone();
                                match crate::mail::secrets::load(&account.email){
                                    Ok(pass)=>{self.mail.testing=Some(account.email.clone());self.mail.note=format!("{}：正在测试连接…",account.email);self.pending.push(Cmd::MailTest(account,pass));}
                                    Err(e)=>self.mail.note=format!("{}：{e}",account.email),
                                }
                            }
                        });
                        settings_label(ui,"添加邮箱");
                        settings_card(ui,|ui|{
                            ui.horizontal(|ui|{
                                ui.add_sized([56.0,30.0],egui::Label::new(egui::RichText::new("邮箱服务").size(13.0)));
                                egui::ComboBox::from_id_salt("mail-provider").width(160.0).selected_text(self.mail.form.provider.label()).show_ui(ui,|ui|{
                                    for p in crate::mail::Provider::ALL{if ui.selectable_label(self.mail.form.provider==p,p.label()).clicked(){self.mail.form.provider=p;}}
                                });
                            });
                            ui.add_space(5.0);
                            ui.label(egui::RichText::new("获取授权码或专用密码").size(12.5).strong());
                            ui.add(egui::Label::new(egui::RichText::new(self.mail.form.provider.hint()).size(12.0).color(Color32::from_gray(200))).wrap());
                            let (label,url)=self.mail.form.provider.setup_link();
                            if ui.link(label).clicked(){self.pending.push(Cmd::OpenUrl(url.into()));}
                            ui.add_space(8.0);
                            let input_width=(ui.available_width()-56.0-48.0-2.0*ui.spacing().item_spacing.x).max(100.0);
                            ui.horizontal(|ui|{
                                ui.add_sized([56.0,30.0],egui::Label::new(egui::RichText::new("邮箱").size(13.0)));
                                ui.add_sized([input_width,30.0],egui::TextEdit::singleline(&mut self.mail.form.email).hint_text("输入邮箱地址"));
                            });
                            ui.horizontal(|ui|{
                                ui.add_sized([56.0,30.0],egui::Label::new(egui::RichText::new("授权码").size(13.0)));
                                ui.add_sized([input_width,30.0],egui::TextEdit::singleline(&mut self.mail.form.password).password(!self.mail.form.show).hint_text("输入邮箱授权码"));
                                if ui.add_sized([48.0,30.0],egui::Button::new(if self.mail.form.show{"隐藏"}else{"显示"})).clicked(){self.mail.form.show=!self.mail.form.show;}
                            });
                            ui.horizontal(|ui|{
                                if ui.button("测试连接").clicked(){
                                    let email=self.mail.form.email.trim().to_string();
                                    if !email.contains('@'){self.mail.note="请先填邮箱地址".into();}
                                    else if self.mail.form.password.is_empty(){self.mail.note="请先填授权码（不是邮箱登录密码）".into();}
                                    else{
                                        self.mail.testing=Some(email.clone());
                                        self.mail.note=format!("{email}：正在测试连接…");
                                        self.pending.push(Cmd::MailTest(crate::mail::Account{email:email.clone(),provider:self.mail.form.provider,enabled:true},self.mail.form.password.clone()));
                                    }
                                }
                                if ui.button("保存并开始监控").clicked(){
                                    let email=self.mail.form.email.trim().to_string();
                                    if !email.contains('@'){self.mail.note="请先填邮箱地址".into();}
                                    else if self.mail.form.password.is_empty(){self.mail.note="请先填授权码（不是邮箱登录密码）".into();}
                                    else{
                                        match crate::mail::secrets::save(&email,&self.mail.form.password){
                                            Ok(())=>{
                                                let account=crate::mail::Account{email:email.clone(),provider:self.mail.form.provider,enabled:true};
                                                if let Some(slot)=self.mail.accounts.iter_mut().find(|a|a.email.eq_ignore_ascii_case(&email)){*slot=account;}else{self.mail.accounts.push(account);}
                                                self.mail.form.password.clear();
                                                self.mail.note=format!("{email}：已保存，正在连接…");
                                                self.mail_accounts_changed();
                                            }
                                            Err(e)=>self.mail.note=e,
                                        }
                                    }
                                }
                            });
                            ui.label(egui::RichText::new("授权码存储在 Windows 凭据管理器中。").size(12.0).color(Color32::from_gray(180)));
                            ui.label(egui::RichText::new("图片中的验证码请在邮箱查看。").size(12.0).color(Color32::from_gray(180)));
                            if !self.mail.note.is_empty(){ui.add_space(3.0);ui.add(egui::Label::new(egui::RichText::new(&self.mail.note).size(12.0).color(Color32::from_gray(180))).wrap());}
                        });
                    }
                    8=>{
                        ui.spacing_mut().item_spacing.y=4.0;
                        // v0.5.6 日程：状态在最上面，后面才是平台、账号和提醒。
                        // 只读；服务器地址由平台决定，用户不用抄。
                        let provider=self.calendar.form.provider;
                        if let Some(cfg)=self.calendar.config.clone(){
                            settings_card(ui,|ui|{
                                let n=self.calendar.events.len();
                                ui.horizontal(|ui|{
                                    ui.label(egui::RichText::new(if self.calendar.connected{format!("● 已连接 · 未来 24 小时 {n} 个日程")}else{format!("● 上次连接失败 · 已知 {n} 个日程")}).size(13.0).strong().color(if self.calendar.connected{Color32::from_rgb(120,220,160)}else{Color32::from_rgb(240,180,110)}));
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|{
                                        if ui.small_button("断开").clicked(){self.calendar.note="已断开，凭据已从 Windows 凭据管理器删除".into();self.calendar.note_error=false;self.pending.push(Cmd::CalendarForget);}
                                        if ui.small_button("立即同步").clicked(){self.calendar.note="正在同步…".into();self.calendar.note_error=false;self.pending.push(Cmd::CalendarSync);}
                                    });
                                });
                                ui.label(egui::RichText::new(format!("{} · 最后同步 {}",cfg.provider.label(),if self.calendar.last_sync.is_empty(){"—".to_string()}else{self.calendar.last_sync.clone()})).size(11.0).color(Color32::from_gray(150)));
                                setting_row(ui,"自动同步","每 5 分钟更新日程；关闭后可点「立即同步」",&mut self.settings.calendar_auto_sync);
                                if self.calendar.failures>=2 && !self.calendar.error.is_empty(){
                                    ui.label(egui::RichText::new(&self.calendar.error).size(12.0).color(Color32::from_rgb(240,96,96)));
                                }
                            });
                        }
                        if ui.add_sized([ui.available_width(),34.0],egui::Button::new("打开日程")).clicked(){self.open_calendar(None);}
                        settings_label(ui,"选择日历平台");
                        settings_card(ui,|ui|{
                            egui::Frame::new().fill(Color32::from_rgb(29,29,31)).corner_radius(8.0).inner_margin(3).show(ui,|ui|{
                                ui.set_min_width(ui.available_width());
                                ui.spacing_mut().item_spacing.x=2.0;
                                ui.horizontal(|ui|{
                                    let width=((ui.available_width()-4.0)/3.0).max(80.0);
                                    for p in crate::calendar::Provider::ALL{
                                        let selected=self.calendar.form.provider==p;
                                        let button=egui::Button::new(egui::RichText::new(p.label()).size(13.0).color(if selected{Color32::WHITE}else{Color32::from_gray(190)}))
                                            .fill(if selected{Color32::from_rgb(81,81,85)}else{Color32::TRANSPARENT})
                                            .stroke(egui::Stroke::NONE).corner_radius(6.0);
                                        if ui.add_sized([width,30.0],button).clicked() && !selected{
                                            self.calendar.form.provider=p;
                                            self.calendar.form.guide_open=false;
                                            self.calendar.note.clear();
                                        }
                                    }
                                });
                            });
                            ui.add_space(7.0);
                            ui.horizontal_wrapped(|ui|{
                                ui.label(egui::RichText::new("服务器").size(12.0).color(Color32::from_gray(180)));
                                ui.label(egui::RichText::new(self.calendar.form.provider.server()).size(12.0).color(Color32::from_gray(205)));
                            });
                        });
                        settings_label(ui,"连接账号");
                        settings_card(ui,|ui|{
                            ui.scope(|ui|{
                                ui.visuals_mut().extreme_bg_color=Color32::from_rgb(25,25,28);
                                let input_width=(ui.available_width()-50.0-48.0-2.0*ui.spacing().item_spacing.x).max(100.0);
                                ui.horizontal(|ui|{
                                    ui.add_sized([50.0,30.0],egui::Label::new(egui::RichText::new("用户名").size(13.0).color(Color32::from_gray(210))));
                                    let user=ui.add_sized([input_width,30.0],egui::TextEdit::singleline(&mut self.calendar.form.username)
                                        .hint_text(egui::RichText::new("输入用户名或粘贴账号信息").color(Color32::from_gray(155))));
                                    if user.changed(){
                                        let pasted=self.calendar.form.username.clone();
                                        if let Some((u,p))=crate::calendar::parse_paste(&pasted){
                                            self.calendar.form.username=u;
                                            self.calendar.form.password=p;
                                            self.calendar.note="已从粘贴内容中填入用户名和密码".into();
                                            self.calendar.note_error=false;
                                        }
                                    }
                                });
                                ui.add_space(5.0);
                                ui.horizontal(|ui|{
                                    ui.add_sized([50.0,30.0],egui::Label::new(egui::RichText::new("密码").size(13.0).color(Color32::from_gray(210))));
                                    ui.add_sized([input_width,30.0],egui::TextEdit::singleline(&mut self.calendar.form.password)
                                        .password(!self.calendar.form.show).hint_text(egui::RichText::new("输入平台生成的密码").color(Color32::from_gray(155))));
                                    if ui.add_sized([48.0,30.0],egui::Button::new(if self.calendar.form.show{"隐藏"}else{"显示"})).clicked(){self.calendar.form.show=!self.calendar.form.show;}
                                });
                            });
                            ui.add_space(7.0);
                            let guide=if self.calendar.form.guide_open{"收起获取方法"}else{"如何获取用户名和密码？"};
                            if ui.link(egui::RichText::new(guide).color(Color32::from_rgb(120,185,245))).clicked(){self.calendar.form.guide_open=!self.calendar.form.guide_open;}
                            if self.calendar.form.guide_open{
                                ui.add_space(4.0);
                                for(i,step)in provider.guide().iter().enumerate(){
                                    ui.label(egui::RichText::new(format!("{}. {}",i+1,step)).size(12.0).color(Color32::from_gray(200)));
                                }
                            }
                            ui.add_space(9.0);
                            let busy=self.calendar.testing;
                            let label=if busy{"正在测试连接…"}else{"测试并保存"};
                            let btn=egui::Button::new(egui::RichText::new(label).size(13.0).strong().color(Color32::WHITE))
                                .fill(if busy{Color32::from_rgb(70,70,76)}else{Color32::from_rgb(45,105,165)})
                                .corner_radius(7.0);
                            if ui.add_enabled_ui(!busy,|ui|ui.add_sized([130.0,32.0],btn)).inner.clicked(){
                                let user=self.calendar.form.username.trim().to_string();
                                if user.is_empty(){self.calendar.note="请先填用户名".into();self.calendar.note_error=true;}
                                else if self.calendar.form.password.is_empty(){self.calendar.note="请先填密码".into();self.calendar.note_error=true;}
                                else{
                                    let cfg=crate::calendar::Config{provider:self.calendar.form.provider,username:user,href:None,server:None};
                                    self.calendar.testing=true;
                                    self.calendar.test_password=self.calendar.form.password.clone();
                                    self.calendar.note="正在连接日历…".into();
                                    self.calendar.note_error=false;
                                    self.pending.push(Cmd::CalendarTest(cfg,self.calendar.form.password.clone()));
                                }
                            }
                            ui.add_space(5.0);
                            ui.label(egui::RichText::new("连接成功后保存，密码存储在 Windows 凭据管理器中。").size(11.0).color(Color32::from_gray(180)));
                            if !self.calendar.note.is_empty(){
                                ui.add_space(5.0);
                                let color=if self.calendar.note_error{Color32::from_rgb(255,125,125)}else{Color32::from_gray(210)};
                                ui.add(egui::Label::new(egui::RichText::new(&self.calendar.note).size(12.0).color(color)).wrap());
                            }
                        });
                        settings_label(ui,"会议提醒");
                        settings_card(ui,|ui|{
                            setting_choices(ui,"提前",&mut self.settings.mail_ics,[5,15,30],"分钟");
                            ui.add_space(5.0);
                            ui.label(egui::RichText::new("与邮件中的会议邀请共用提醒，同一场会议只提醒一次。").size(11.0).color(Color32::from_gray(180)));
                        });
                    }
                    _=>{
                        settings_label(ui,"岛仔 Daozai");
                        settings_card(ui,|ui|{
                            ui.label(egui::RichText::new(format!("版本 {}",env!("CARGO_PKG_VERSION"))).size(13.0));
                            self.updates.ui(ui);
                            ui.label(egui::RichText::new("桌面顶部的音乐、专注、提醒与日程小助手。").size(12.0).color(Color32::from_gray(180)));
                            ui.horizontal_wrapped(|ui|{
                                ui.label(egui::RichText::new("设备图标：Solar Icons · 480 Design").size(11.0).color(Color32::from_gray(150)));
                                ui.hyperlink_to("CC BY 4.0", "https://creativecommons.org/licenses/by/4.0/");
                            });
                            ui.add_space(5.0);
                            if ui.add_sized([150.0,32.0],egui::Button::new("打开本地数据目录")).clicked(){self.pending.push(Cmd::Open(crate::model::config_dir()));}
                        });
                    }
                    }
                });
            });
            if before!=format!("{:?}",self.settings){self.settings.save();}
        });
    }
}
