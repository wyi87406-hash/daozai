//! 无窗口交互测试：直接向 egui 注入输入事件，验证悬停展开、切页、按钮。
use crate::app::{App, Env, Options, PH, PW, WIN_H, WIN_W};
use eframe::egui::{self, Pos2, Rect, Vec2};
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn switching_music_source_discards_previous_lyrics() {
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    let mut h=H::new();
    h.media=crate::smtc::NowPlaying{source:"cloudmusic.exe".into(),title:"Track".into(),artist:"Artist".into(),duration:120.0,..Default::default()};
    h.frame(vec![]);
    h.app.set_lyrics("Track","Artist",120.0,crate::lyrics::parse_lrc("[00:01]旧来源歌词"));
    h.media.source="汽水音乐".into();
    h.frame(vec![]);
    assert!(h.app.ly.lyrics.is_none(),"同一首歌切换播放器后必须重新选择歌词来源");
}

#[test]
fn settings_keep_dark_buttons_when_system_theme_changes() {
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    let h=H::new();
    for theme in [egui::Theme::Light,egui::Theme::Dark,egui::Theme::Light] {
        let _=h.ctx.run(egui::RawInput{system_theme:Some(theme),..Default::default()},|ctx| {
            assert!(ctx.style().visuals.widgets.inactive.weak_bg_fill.r()<100,"系统浅色模式不能把深色设置里的按钮变成白底");
        });
    }
}

#[test]
fn expanded_settings_routes_preserve_inline_controls(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    for (page,tab) in [1,2,3,5,4,7,11,8].into_iter().enumerate(){
        let mut h=H::new();h.app.show_page(page);h.hover(H::local(200.0,40.0),1.0);
        h.click(H::local(394.0,18.0));
        assert!(h.app.settings_open,"页面 {page} 应打开设置窗口");
        assert_eq!(h.app.settings_tab,tab);assert_eq!(h.app.page,page);
        assert!(h.app.subpage.is_none());
    }
    for (page,point) in [(1,H::local(111.0,18.0)),(2,H::local(149.0,110.0))]{
        let mut h=H::new();h.app.show_page(page);h.hover(H::local(200.0,40.0),1.0);
        h.click(point);assert_eq!(h.app.subpage,Some(page));assert!(!h.app.settings_open);
        h.click(H::local(394.0,18.0));assert!(h.app.settings_open);
        assert_eq!(h.app.subpage,Some(page),"打开完整设置后应保留岛内调整页");
    }
    let mut h=H::new();h.app.show_page(3);h.hover(H::local(200.0,40.0),1.0);
    h.click(H::local(330.0,18.0));assert!(h.cmds.iter().any(|cmd|matches!(cmd,crate::app::Cmd::AddFiles)));
    assert!(!h.app.settings_open);
    h.app.show_page(6);h.hover(H::local(200.0,40.0),1.0);h.click(H::local(319.0,18.0));
    assert!(h.cmds.iter().any(|cmd|matches!(cmd,crate::app::Cmd::QuotaRefresh)));assert!(!h.app.settings_open);
}

#[test]
fn quality_late_calendar_response_does_not_restore_disconnected_account() {
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    let config=crate::calendar::Config{provider:crate::calendar::Provider::Feishu,username:"old".into(),href:None,server:None};
    h.app.forget_calendar();
    h.app.calendar_event(crate::calendar::Event::Synced{config:config.clone(),events:vec![],at:"12:00".into()});
    assert!(!h.app.calendar.connected && h.app.calendar.last_sync.is_empty());
    h.app.calendar.config=Some(crate::calendar::Config{username:"new".into(),..config.clone()});
    h.app.calendar.connected=true;
    h.app.calendar_event(crate::calendar::Event::Failed{config,message:"旧连接失败".into()});
    assert!(h.app.calendar.connected && h.app.calendar.error.is_empty());
}

#[test]
fn quality_distinct_reminders_are_queued_instead_of_overwritten() {
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    let meeting=crate::mail::Meeting{ all_day:false,title:"会议一".into(),start:chrono::Local::now().timestamp()+600,end:None,at:String::new(),uid:"one".into()};
    h.app.meeting_alert(meeting.clone(),"飞书".into(),7);
    h.app.meeting_alert(crate::mail::Meeting{ all_day:false,title:"会议二".into(),uid:"two".into(),..meeting.clone()},"飞书".into(),7);
    assert_eq!(h.app.alerts.len(),2,"同时到期的两场会议都需要提醒");
    h.app.meeting_alert(meeting,"飞书".into(),7);assert_eq!(h.app.alerts.len(),2,"同一会议重复更新应合并");
    h.app.alert("待办到时间了","任务一",4,true);h.app.alert("待办到时间了","任务二",4,true);
    assert_eq!(h.app.alerts.iter().filter(|a|a.title=="待办到时间了").count(),2);
    h.app.alert("系统音量","10%",0,false);h.app.alert("系统音量","20%",0,false);
    assert_eq!(h.app.alerts.iter().filter(|a|a.title=="系统音量").count(),1);
}

#[test]
fn quality_removed_mail_accounts_ignore_in_flight_results() {
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    h.app.mail_event(crate::mail::Event::Status{account:"removed@example.com".into(),status:crate::mail::Status::Ready});
    h.app.mail_event(crate::mail::Event::Mail{item:crate::mail::Item{account:"removed@example.com".into(),from:"发件人".into(),subject:"延迟邮件".into(),time:String::new(),code:None,summary:String::new(),links:vec![],meeting:None,uid:1},new:true});
    assert!(h.app.mail.status.is_empty() && h.app.mail.recent.is_empty());
    assert!(h.app.notification_bell_until<=h.t);
}

struct H {
    ctx: egui::Context,
    app: App,
    t: f64,
    cursor: Option<Pos2>,
    cmds:Vec<crate::app::Cmd>,
    region:Rect,
    hit:Rect,
    animating:bool,
    shapes:Vec<egui::epaint::ClippedShape>,
    bt: Vec<crate::bt::Device>,
    media: crate::smtc::NowPlaying,
    power: (Option<u8>, bool),
    network: Option<bool>,
    system_idle: Option<f64>,
    browser_at_top: Option<bool>,
    file_drop_zone:Option<Rect>,
    hovered_files:Vec<egui::HoveredFile>,
    dropped_files:Vec<egui::DroppedFile>,
}

impl H {
    fn new() -> Self {
        let dir = std::env::temp_dir().join(format!("daozai-test-{}-{}", std::process::id(), rand_id()));
        std::env::set_var("DAOZAI_DATA_DIR", &dir);
        let ctx = egui::Context::default();
        let app = App::new(&ctx, Options { fetch_lyrics: false, pet_idle: 1e9 });
        Self { ctx, app, t: 0.0, cursor: None,cmds:vec![],region:Rect::NOTHING,hit:Rect::NOTHING,animating:false,shapes:Vec::new(), bt:Vec::new(),media:Default::default(),power:(None,false),network:None,system_idle:None,browser_at_top:None,file_drop_zone:None,hovered_files:vec![],dropped_files:vec![] }
    }
    fn frame(&mut self, events: Vec<egui::Event>) {
        self.t += 1.0 / 60.0;
        let raw = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(crate::app::CANVAS_W, WIN_H))),
            time: Some(self.t),
            events,
            hovered_files:self.hovered_files.clone(),
            dropped_files:std::mem::take(&mut self.dropped_files),
            focused: true,
            ..Default::default()
        };
        let env = Env {
            t: self.t,
            clock: "09:41:00".into(),
            today: "2026-10-28".into(),
            date: "10月28日 周三".into(),
            cursor: self.cursor,
            bt: self.bt.clone(),
            bt_ready: true,
            media:self.media.clone(),
            battery:self.power.0,
            charging:self.power.1,
            network:self.network,
            idle_seconds:self.system_idle,
            browser_at_top:self.browser_at_top,
            ..Default::default()
        };
        let cmds=&mut self.cmds;
        let region=&mut self.region;
        let hit=&mut self.hit;
        let animating=&mut self.animating;
        let app = &mut self.app;
        let file_drop_zone=&mut self.file_drop_zone;
        let output = self.ctx.run(raw, |ctx| {
            egui::CentralPanel::default().frame(egui::Frame::new()).show(ctx, |ui| {
                let win = Rect::from_min_size(ui.max_rect().min,Vec2::new(WIN_W,WIN_H));
                let frame=app.frame(ui, win, &env);*region=frame.region;*hit=frame.hit;*animating=frame.animating;*file_drop_zone=frame.file_drop_zone;cmds.extend(frame.cmds);
            });
        });
        self.shapes=output.shapes;
    }
    fn hover(&mut self, p: Pos2, secs: f64) {
        self.cursor = Some(p);
        let n = (secs * 60.0) as usize;
        for _ in 0..n {
            self.frame(vec![egui::Event::PointerMoved(p)]);
        }
    }
    fn click(&mut self, p: Pos2) {
        self.cursor = Some(p);
        self.frame(vec![egui::Event::PointerMoved(p)]);
        self.frame(vec![egui::Event::PointerButton { pos: p, button: egui::PointerButton::Primary, pressed: true, modifiers: Default::default() }]);
        self.frame(vec![egui::Event::PointerButton { pos: p, button: egui::PointerButton::Primary, pressed: false, modifiers: Default::default() }]);
    }
    fn click_text(&mut self,label:&str){
        if !self.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Text(t)if t.galley.text()==label))&&self.app.notice.is_some(){self.click_notice_badge();}
        let point=self.shapes.iter().find_map(|s|match &s.shape{egui::Shape::Text(t)if t.galley.text()==label&&t.opacity_factor>0.9&&s.clip_rect.contains(t.visual_bounding_rect().center())=>Some(t.visual_bounding_rect().center()),_=>None}).unwrap_or_else(||panic!("未找到可点击文字：{label}"));
        self.click(point);
    }
    fn click_notice_badge(&mut self){if self.app.compact_notices(){self.click(Pos2::new(WIN_W/2.0+(self.app.current_width()-44.0)/2.0+26.0,15.0));}}
    fn wheel(&mut self,delta:f32) {
        self.frame(vec![egui::Event::MouseWheel{unit:egui::MouseWheelUnit::Point,delta:Vec2::new(0.0,delta),modifiers:Default::default()}]);
    }
    fn picker_frame(&mut self,events:Vec<egui::Event>){
        self.frame(events);
    }
    fn picker_click(&mut self,label:&str){
        for _ in 0..60 {self.picker_frame(vec![]);if (self.app.progress()-1.0).abs()<0.01 {break;}}
        self.picker_frame(vec![]);
        let find=|h:&H|h.shapes.iter().find_map(|shape|match &shape.shape{
            egui::Shape::Text(text) if text.galley.text()==label && shape.clip_rect.contains(text.visual_bounding_rect().center())=>Some(text.visual_bounding_rect().center()),_=>None,
        });
        let point=find(self).unwrap_or_else(||panic!("未找到可见选项：{label}"));
        assert!(self.hit.contains(point),"选项 {label} 超出岛体命中区域");
        self.picker_frame(vec![egui::Event::PointerMoved(point)]);
        for pressed in [true,false]{self.picker_frame(vec![egui::Event::PointerButton{pos:point,button:egui::PointerButton::Primary,pressed,modifiers:Default::default()}]);}
    }
    fn local(x: f32, y: f32) -> Pos2 {
        Pos2::new((WIN_W - PW) / 2.0 + x, y)
    }
    fn quota_accounts(&mut self,count:usize) {
        use crate::quota::{Account,Provider,Snapshot,Window};
        for i in 0..count {
            let id=format!("preview-{i}");
            self.app.quota.config.accounts.push(Account{id:id.clone(),provider:Provider::ALL[i%3],alias:format!("账号 {}",i+1),source:format!("preview-{i}.json").into(),slot:None,enabled:true,pinned:true});
            self.app.quota_event(crate::quota::Event{id,result:Ok(Snapshot{windows:vec![Window{label:"本周".into(),seconds:Some(604800),remaining:100.0,reset:None,text:None}],..Default::default()})});
        }
    }
    fn quota_centers(&self)->Vec<f32> {
        fn collect(shape:&egui::Shape,out:&mut Vec<f32>) {
            match shape {
                egui::Shape::Vec(shapes)=>for shape in shapes {collect(shape,out);},
                egui::Shape::Path(path) if path.points.len()>50=>{
                    let min=path.points.iter().map(|p|p.x).fold(f32::INFINITY,f32::min);
                    let max=path.points.iter().map(|p|p.x).fold(f32::NEG_INFINITY,f32::max);
                    if (20.0..28.0).contains(&(max-min)) {out.push((min+max)/2.0);}
                },
                egui::Shape::Circle(circle) if (10.0..14.0).contains(&circle.radius) && circle.stroke.width>0.0 && (circle.center.y-15.0).abs()<0.5=>out.push(circle.center.x),
                _=>{},
            }
        }
        let mut out=Vec::new();for shape in &self.shapes {collect(&shape.shape,&mut out);}
        out.sort_by(f32::total_cmp);out
    }
}

fn rand_id() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos() as u64
}
#[test]
fn shelf_drop_zone_leaves_normal_hover_and_text_selection_unchanged(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    for _ in 0..90{h.frame(vec![]);}let zone=h.file_drop_zone.unwrap();
    assert_eq!(zone.width(),PW);assert_eq!(zone.height(),96.0);
    let point=zone.right_bottom()-Vec2::splat(12.0);assert!(!h.hit.contains(point));
    h.hover(point,0.8);assert!(!h.app.open);
    h.frame(vec![egui::Event::PointerButton{pos:point,button:egui::PointerButton::Primary,pressed:true,modifiers:Default::default()}]);
    h.hover(point,0.8);assert!(!h.app.open,"按住鼠标拖选文字不应打开置物架");
    h.frame(vec![egui::Event::PointerButton{pos:point,button:egui::PointerButton::Primary,pressed:false,modifiers:Default::default()}]);
    h.app.settings.visible_pages[3]=false;h.frame(vec![]);assert!(h.file_drop_zone.is_none());
}

#[test]
fn shelf_file_drag_opens_below_the_island_and_cancellation_restores_the_page(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    h.app.go(1,1.0);for _ in 0..90{h.frame(vec![]);}
    let point=h.file_drop_zone.unwrap().right_bottom()-Vec2::splat(12.0);
    assert!(!h.hit.contains(point));h.cursor=Some(point);
    h.hovered_files.push(egui::HoveredFile{path:Some(crate::model::config_dir().join("拖入文件.txt")),..Default::default()});
    h.frame(vec![]);assert!(h.app.open && h.app.page==3 && h.app.drag_hot,"文件进入感应区应立即展开置物架");
    for _ in 0..60{h.frame(vec![]);}assert!(h.app.open);
    h.hovered_files.clear();h.cursor=None;for _ in 0..60{h.frame(vec![]);}
    assert!(!h.app.drag_hot && !h.app.open);assert_eq!(h.app.page,1);
    assert!(h.app.shelf.items.is_empty(),"取消拖入不应保存文件");
}

#[test]
fn shelf_quick_drop_in_the_capture_zone_shows_the_added_file(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();h.frame(vec![]);
    let path=crate::model::config_dir().join("快速拖入.txt");std::fs::create_dir_all(path.parent().unwrap()).unwrap();std::fs::write(&path,"test").unwrap();
    h.cursor=Some(h.file_drop_zone.unwrap().right_bottom()-Vec2::splat(12.0));
    h.dropped_files.push(egui::DroppedFile{path:Some(path.clone()),..Default::default()});
    for _ in 0..90{h.frame(vec![]);}
    assert!(h.app.open && h.app.page==3);assert_eq!(h.app.shelf.items,vec![path.clone()]);assert!(path.exists());
    let mut h=H::new();for _ in 0..90{h.frame(vec![]);}
    h.cursor=Some(h.file_drop_zone.unwrap().right_bottom()-Vec2::splat(12.0));h.app.settings.visible_pages[3]=false;
    h.dropped_files.push(egui::DroppedFile{path:Some(path),..Default::default()});h.frame(vec![]);
    assert!(!h.app.open && !h.app.settings_open,"关闭置物架后应忽略拖入");
}

#[test]
fn settings_opening_restores_focus_once_without_becoming_topmost() {
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    let mut h=H::new();
    h.ctx.set_embed_viewports(false);
    let frame=|h:&mut H,focused:bool| {
        egui::Context::set_immediate_viewport_renderer(move |ctx,mut viewport| {
            let id=viewport.ids.this;
            let mut raw=egui::RawInput{viewport_id:id,focused,..Default::default()};
            raw.viewports.insert(id,egui::ViewportInfo{focused:Some(focused),minimized:Some(!focused),..Default::default()});
            let _=ctx.run(raw,|ctx|(viewport.viewport_ui_cb)(ctx));
        });
        let id=egui::ViewportId::from_hash_of("daozai-settings");
        let mut raw=egui::RawInput::default();
        raw.viewports.insert(id,egui::ViewportInfo{focused:Some(focused),..Default::default()});
        let mut output=h.ctx.run(raw,|ctx|h.app.settings_ui(ctx,false));
        let viewport=output.viewport_output.remove(&id).unwrap();
        (viewport.builder,viewport.commands)
    };
    h.app.open_mail_settings();
    let (builder,commands)=frame(&mut h,false);
    assert_eq!(builder.window_level,Some(egui::WindowLevel::Normal),"打开设置时应保持普通窗口层级");
    assert!(commands.iter().any(|cmd|matches!(cmd,egui::ViewportCommand::Minimized(false))),"应恢复已最小化的设置窗口：{commands:?}");
    assert!(commands.iter().any(|cmd|matches!(cmd,egui::ViewportCommand::Focus)),"应将焦点交给设置窗口");
    let (builder,commands)=frame(&mut h,true);
    assert_eq!(builder.window_level,Some(egui::WindowLevel::Normal),"操作设置时也不能切换到置顶层");
    assert!(!commands.iter().any(|cmd|matches!(cmd,egui::ViewportCommand::Focus)),"正常绘制不应反复抢焦点");
    let (builder,commands)=frame(&mut h,false);
    assert_eq!(builder.window_level,Some(egui::WindowLevel::Normal),"切回其他程序后应解除设置置顶");
    assert!(!commands.iter().any(|cmd|matches!(cmd,egui::ViewportCommand::Focus)));
    h.app.open_mail_settings();
    let (builder,commands)=frame(&mut h,false);
    assert_eq!(builder.window_level,Some(egui::WindowLevel::Normal),"再次唤起设置仍须保持普通窗口层级");
    assert!(commands.iter().any(|cmd|matches!(cmd,egui::ViewportCommand::Minimized(false))));
    assert!(commands.iter().any(|cmd|matches!(cmd,egui::ViewportCommand::Focus)),"设置已打开时仍应响应再次唤起");
}

#[test]
fn web_video_is_hidden_without_changing_music_preferences(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    let mut h=H::new();
    h.app.opts.fetch_lyrics=true;h.app.show_lyrics=true;
    h.app.ly.key="Previous Song|Artist|120".into();
    h.app.ly.lyrics=Some(crate::lyrics::parse_lrc("[00:01.00]previous song"));
    h.app.ly.error=Some(crate::lyrics::FetchError::Unavailable("503".into()));
    let (tx,rx)=std::sync::mpsc::channel();h.app.ly.rx=Some(rx);
    h.media=crate::smtc::NowPlaying{title:"小唐的Token的抖音 - 抖音".into(),playing:true,duration:17.0,..Default::default()};
    h.frame(vec![]);
    assert!(h.app.media.title.is_empty() && !h.app.media.playing,"视频标题和播放状态不应进入岛内音乐页面");
    assert!(h.app.ly.key.is_empty(),"视频不应建立歌曲取词任务");
    assert!(h.app.ly.lyrics.is_none() && h.app.ly.error.is_none(),"切到视频应清除上一首歌词和歌词错误");
    assert!(tx.send(Ok(crate::lyrics::parse_lrc("[00:01.00]late song"))).is_err(),"视频不能接收旧歌曲迟到的查询结果");
    h.hover(Pos2::new(WIN_W/2.0,12.0),1.0);
    h.click(H::local(74.0,118.0));
    assert!(h.app.show_lyrics,"视频页面的标签不应切换用户的歌曲歌词偏好");
    h.click(H::local(210.5,118.0));
    assert!(!h.cmds.iter().any(|cmd|matches!(cmd,crate::app::Cmd::Media(_))),"音乐页不应控制被隐藏的视频");
    assert!(!matches!(h.app.pet_state(),crate::extras::PetState::Music),"视频不应触发听歌表情");
    h.app.opts.fetch_lyrics=false;
    h.media=crate::smtc::NowPlaying{title:"Track".into(),artist:"Artist".into(),duration:120.0,..Default::default()};
    h.hover(Pos2::new(WIN_W/2.0,12.0),0.5);
    h.click(H::local(74.0,118.0));
    assert!(!h.app.show_lyrics,"回到歌曲时应恢复原来的词按钮");
}

#[test]
fn web_video_does_not_override_the_minimal_island(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    let mut h=H::new();h.app.settings.minimal=true;
    h.media=crate::smtc::NowPlaying{title:"小唐的Token的抖音 - 抖音".into(),playing:true,duration:17.0,..Default::default()};
    for _ in 0..90 {h.frame(vec![]);}
    assert!(h.region.height()<20.0,"视频不能触发歌曲专用的收起态");
}

#[test]
fn minimal_mode_stays_a_line_with_music_and_after_media_changes(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    let mut h=H::new();
    h.app.settings.minimal=true;
    h.media=crate::smtc::NowPlaying{title:"Track".into(),artist:"Artist".into(),playing:true,duration:120.0,..Default::default()};
    for _ in 0..90 {h.frame(vec![]);}
    assert!(h.region.height()<20.0,"播放音乐时仍应遵守极简开关");
    assert!(h.hit.height()<=crate::app::BAR_H_MIN+0.1,"鼠标命中范围也应收回细线");
    h.media.title="Another Track".into();h.media.playing=false;
    for _ in 0..90 {h.frame(vec![]);}
    assert!(h.region.height()<20.0,"暂停或切歌不能覆盖极简开关");
    h.media=Default::default();
    for _ in 0..90 {h.frame(vec![]);}
    assert!(h.region.height()<20.0,"没有媒体时恢复用户选择的极简状态");
}

#[test]
fn toggling_minimal_changes_the_island_while_media_is_present(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    h.media=crate::smtc::NowPlaying{title:"Track".into(),artist:"Artist".into(),playing:true,duration:120.0,..Default::default()};
    for _ in 0..90 {h.frame(vec![]);}assert!(h.region.height()>30.0);
    h.app.settings.minimal=true;for _ in 0..90 {h.frame(vec![]);}
    assert!(h.region.height()<20.0,"开启极简后应收成细线");
    h.app.settings.minimal=false;for _ in 0..90 {h.frame(vec![]);}
    assert!(h.region.height()>30.0,"关闭极简后应恢复普通岛体");
}

#[test]
fn minimal_mode_keeps_focus_running_and_returns_after_notifications(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    h.app.settings.minimal=true;h.app.pomo.running=true;h.app.pomo.remaining=120.0;
    for _ in 0..90 {h.frame(vec![]);}assert!(h.region.height()<20.0,"专注不能覆盖极简开关");
    h.app.alert("系统音量","音量 50%",5,false);
    for _ in 0..60 {h.frame(vec![]);}assert!(h.region.height()>30.0,"临时提醒仍应可见");
    for _ in 0..240 {h.frame(vec![]);}assert!(h.region.height()<20.0,"提醒结束后应回到细线");
    assert!(h.app.pomo.running&&h.app.pomo.remaining<115.0,"极简仅改变显示，计时继续运行");
}

#[test]
fn delayed_media_samples_keep_the_live_position_and_follow_pause_and_seek(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    h.media=crate::smtc::NowPlaying{title:"Track".into(),artist:"Artist".into(),position:9.2,duration:120.0,playing:true,sampled_at:Some(std::time::Instant::now()-std::time::Duration::from_millis(600)),..Default::default()};
    h.frame(vec![]);
    assert!(h.app.media.position>=9.79,"媒体位置必须包含送到界面之前经过的时间");
    h.media.playing=false;h.media.position=9.4;h.frame(vec![]);
    assert_eq!(h.app.media.position,9.4,"暂停不能继续外推");
    h.media.playing=true;h.media.position=4.0;h.media.sampled_at=Some(std::time::Instant::now());h.frame(vec![]);
    assert!((4.0..4.2).contains(&h.app.media.position),"跳转后应使用新的播放位置");
    h.media.position=119.9;h.media.sampled_at=Some(std::time::Instant::now()-std::time::Duration::from_secs(1));h.frame(vec![]);
    assert_eq!(h.app.media.position,120.0,"外推不能超过歌曲结尾");
}

#[test]
fn kugou_position_advances_without_duration_and_respects_pause_and_seek(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    h.media=crate::smtc::NowPlaying{source:"kugou".into(),title:"Track".into(),artist:"Artist".into(),position:42.0,duration:0.0,playing:true,sampled_at:Some(std::time::Instant::now()-std::time::Duration::from_millis(600)),..Default::default()};
    h.frame(vec![]);
    assert!(h.app.media.position>=42.59,"未知总时长不能阻止真实进度外推");
    h.media.playing=false;h.frame(vec![]);assert_eq!(h.app.media.position,42.0);
    h.media.playing=true;h.media.position=12.0;h.media.sampled_at=Some(std::time::Instant::now());h.frame(vec![]);
    assert!((12.0..12.2).contains(&h.app.media.position),"拖动后必须跟随新的实际位置");
}

#[test]
fn music_speaker_button_toggles_mute(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    let mut h=H::new();
    h.media=crate::smtc::NowPlaying{title:"Track".into(),artist:"Artist".into(),duration:120.0,..Default::default()};
    h.app.volume_ui=Some((45,false));
    h.hover(Pos2::new(WIN_W/2.0,12.0),1.0);
    h.click(H::local(367.0,118.0));
    assert!(h.cmds.contains(&crate::app::Cmd::ToggleMute));
    assert_eq!(h.app.volume_ui,Some((45,true)));
}

#[test]
fn music_uses_system_artwork_and_clears_it_when_the_next_song_has_none(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    let mut h=H::new();
    let image=std::sync::Arc::new(egui::ColorImage::new([32,32],egui::Color32::BLUE));
    h.media=crate::smtc::NowPlaying{title:"First".into(),artist:"Artist".into(),artwork:Some(image.clone()),..Default::default()};
    h.hover(Pos2::new(WIN_W/2.0,12.0),1.0);
    let (cached,texture)=h.app.music_artwork.as_ref().expect("播放页应使用系统封面");
    assert!(std::sync::Arc::ptr_eq(cached,&image));
    let texture_id=texture.id();
    h.frame(vec![]);
    assert_eq!(h.app.music_artwork.as_ref().unwrap().1.id(),texture_id,"进度更新不应重复上传纹理");
    h.media=crate::smtc::NowPlaying{title:"Second".into(),artist:"Artist".into(),..Default::default()};
    h.frame(vec![]);
    assert!(h.app.music_artwork.is_none(),"缺少封面的下一首歌应恢复占位图");
}

#[test]
fn failed_lyrics_can_be_retried_from_the_music_page(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    let mut h=H::new();
    h.media=crate::smtc::NowPlaying{title:"Track".into(),artist:"Artist".into(),duration:120.0,..Default::default()};
    h.app.ly.key="Track|Artist|120".into();
    h.app.ly.error=Some(crate::lyrics::FetchError::Unavailable("HTTP 502".into()));
    h.hover(Pos2::new(WIN_W/2.0,12.0),1.0);
    h.click(H::local(310.0,68.0));
    assert!(h.app.ly.key.is_empty(),"点击错误提示后应允许重新请求当前歌曲");
}

#[test]
fn transient_lyric_failures_retry_but_missing_songs_do_not_loop(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    for (error,retry) in [(crate::lyrics::FetchError::Unavailable("503".into()),true),(crate::lyrics::FetchError::NotFound,false),(crate::lyrics::FetchError::IncompleteMetadata,false)] {
        let mut h=H::new();
        h.media=crate::smtc::NowPlaying{title:"Track".into(),artist:"Artist".into(),duration:120.0,..Default::default()};
        h.app.opts.fetch_lyrics=true;h.app.show_lyrics=true;
        h.app.ly.key="Track|Artist|120".into();
        let (tx,rx)=std::sync::mpsc::channel();
        h.app.ly.rx=Some(rx);h.app.ly.loading=true;
        tx.send(Err(error.clone())).unwrap();
        h.frame(vec![]);
        assert_eq!(h.app.ly.error,Some(error));
        assert_eq!(h.app.ly.retry_at.is_some(),retry);
        assert!(!h.app.ly.loading);
        if let Some(at)=h.app.ly.retry_at {assert!((at-h.t-30.0).abs()<0.001,"网络恢复重试应有间隔");}
    }
}

#[test]
fn music_lyrics_show_in_a_separate_right_column() {
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    let mut h=H::new();
    h.media=crate::smtc::NowPlaying{title:"Track".into(),artist:"Artist".into(),position:7.0,duration:120.0,..Default::default()};
    h.app.set_lyrics("Track","Artist",120.0,crate::lyrics::parse_lrc("[00:01]上一句\n[00:05]当前歌词\n[00:10]下一句"));
    h.hover(Pos2::new(WIN_W/2.0,12.0),1.0);
    let current=h.shapes.iter().find_map(|shape|match &shape.shape {
        egui::Shape::Text(text) if text.galley.text()=="当前歌词"=>Some(text.pos),_=>None,
    }).expect("展开页应显示同步歌词");
    assert!(current.x>=H::local(243.0,0.0).x,"歌词必须位于播放器右侧");
    let title=h.shapes.iter().find_map(|shape|match &shape.shape {
        egui::Shape::Text(text) if text.galley.text()=="Track"=>Some(text.pos),_=>None,
    }).unwrap();
    assert!(title.x<H::local(233.0,0.0).x);
    h.click(H::local(74.0,118.0));
    assert!(!h.app.show_lyrics);
    h.frame(vec![]);
    assert!(!h.shapes.iter().any(|shape|matches!(&shape.shape,egui::Shape::Text(text) if text.galley.text()=="当前歌词")));
}

#[test]
fn music_lyrics_island_toggle_is_independent_and_persistent() {
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    let mut h=H::new();
    h.app.show_lyrics=false;h.app.settings.lyrics_enabled=false;
    h.media=crate::smtc::NowPlaying{title:"Track".into(),artist:"Artist".into(),position:7.0,duration:120.0,..Default::default()};
    h.app.set_lyrics("Track","Artist",120.0,crate::lyrics::parse_lrc("[00:05]当前歌词\n[00:10]下一句"));
    h.hover(Pos2::new(WIN_W/2.0,12.0),1.0);
    h.click(H::local(313.0,118.0));
    assert!(h.app.settings.island_lyrics);
    assert!(!h.app.show_lyrics && !h.app.settings.lyrics_enabled);
    let saved=crate::model::Settings::load();
    assert!(saved.island_lyrics && !saved.lyrics_enabled);
    h.hover(Pos2::new(WIN_W/2.0,PH+30.0),1.0);
    assert!(!h.app.open);
    assert_eq!(h.app.island_lyric().map(|(_,line)|line),Some("当前歌词"));
    let pet_width=if h.app.pet_enabled(){48.0}else{0.0};
    assert!((h.app.current_width()-crate::app::LYRIC_BAR_W-pet_width).abs()<2.0);
}

#[test]
fn music_lyric_timing_follows_saved_delay_and_preserves_playback_position(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    let mut h=H::new();
    h.media=crate::smtc::NowPlaying{title:"Track".into(),artist:"Artist".into(),position:9.8,duration:120.0,..Default::default()};
    h.app.set_lyrics("Track","Artist",120.0,crate::lyrics::parse_lrc("[00:00]第一句\n[00:05]当前歌词\n[00:10]下一句"));
    h.frame(vec![]);
    assert_eq!(h.app.settings.lyric_delay_ms,-350,"未设置时应保留原有提前 0.35 秒的预设");
    assert!((h.app.lyric_position()-10.15).abs()<0.001,"默认应提前 0.35 秒显示，且只应用一次");
    for (delay,position,expected) in [(500,10.2,Some("当前歌词")),(-500,9.8,Some("下一句")),(0,9.8,Some("当前歌词")),(500,0.2,None),(500,0.6,Some("第一句"))]{
        std::fs::write(crate::model::config_dir().join("settings.txt"),format!("lyric_delay_ms={delay}\nisland_lyrics=1\n")).unwrap();
        h.app.settings=crate::model::Settings::load();
        h.media.position=position;h.frame(vec![]);
        assert_eq!(h.app.island_lyric().map(|(_,line)|line),expected,"延时 {delay} ms，播放位置 {position}");
        assert!((h.app.media.position-position).abs()<0.001,"调整歌词不应改变播放进度");
        h.hover(Pos2::new(WIN_W/2.0,12.0),1.0);
        for text in ["第一句","当前歌词","下一句"]{
            let active=h.shapes.iter().any(|shape|matches!(&shape.shape,egui::Shape::Text(t) if t.galley.text()==text && t.galley.job.sections[0].format.font_id.size==12.0));
            assert_eq!(active,expected==Some(text),"展开歌词也应使用相同的显示时间");
        }
        h.app.settings.save();
        assert_eq!(crate::model::Settings::load().lyric_delay_ms,delay,"重启后应保留时间调整");
    }
    for (input,expected) in [("invalid",-350),("999999",5000),("-999999",-5000),("",-350)]{
        std::fs::write(crate::model::config_dir().join("settings.txt"),format!("lyric_delay_ms={input}\n")).unwrap();
        assert_eq!(crate::model::Settings::load().lyric_delay_ms,expected);
    }
}

#[test]
fn music_lyric_timing_controls_match_focus_save_and_fit_the_small_settings_window(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    let mut h=H::new();h.app.settings_tab=1;h.app.open_settings();h.ctx.set_embed_viewports(false);
    fn frame(h:&mut H,events:Vec<egui::Event>)->Vec<egui::epaint::ClippedShape>{
        let output=std::sync::Arc::new(Mutex::new(None));let capture=output.clone();
        egui::Context::set_immediate_viewport_renderer(move|ctx,mut viewport|{
            let id=viewport.ids.this;
            let raw=egui::RawInput{viewport_id:id,screen_rect:Some(Rect::from_min_size(Pos2::ZERO,egui::vec2(600.0,450.0))),
                viewports:std::iter::once((id,egui::ViewportInfo{native_pixels_per_point:Some(1.25),focused:Some(true),..Default::default()})).collect(),
                focused:true,events:events.clone(),..Default::default()};
            *capture.lock().unwrap()=Some(ctx.run(raw,|ctx|(viewport.viewport_ui_cb)(ctx)));
        });
        let raw=egui::RawInput{viewports:std::iter::once((egui::ViewportId::ROOT,egui::ViewportInfo{native_pixels_per_point:Some(1.25),..Default::default()})).collect(),..Default::default()};
        let _=h.ctx.run(raw,|ctx|h.app.settings_ui(ctx,false));
        let child=output.lock().unwrap().take().unwrap();child.shapes
    }
    let text_rect=|shapes:&[egui::epaint::ClippedShape],text:&str|shapes.iter().find_map(|shape|match &shape.shape{
        egui::Shape::Text(t)if t.galley.text()==text=>Some(t.visual_bounding_rect()),_=>None,
    }).unwrap_or_else(||panic!("设置中缺少 {text}"));
    for _ in 0..3{frame(&mut h,vec![]);}
    let music=frame(&mut h,vec![]);
    text_rect(&music,"0.35 秒");
    assert_eq!(h.app.settings.lyric_delay_ms,-350,"仅打开设置不能将 0.35 秒舍入为 0.4 秒");
    for button in ["−","+"]{text_rect(&music,button);}
    h.app.settings_tab=2;
    for _ in 0..3{frame(&mut h,vec![]);}
    let focus=frame(&mut h,vec![]);
    for button in ["−","+"]{
        let a=text_rect(&music,button);let b=text_rect(&focus,button);
        assert!((a.left()-b.left()).abs()<0.1 && (a.width()-b.width()).abs()<0.1 && (a.height()-b.height()).abs()<0.1,"歌词和专注应共用同样的数值控件及右侧对齐");
    }
    h.app.settings_tab=1;
    for _ in 0..3{frame(&mut h,vec![]);}
    for (button,expected) in [("+",-450),("−",-350),("−",-250),("+",-350)]{
        let shapes=frame(&mut h,vec![]);
        let rect=text_rect(&shapes,button);
        assert!(Rect::from_min_max(egui::pos2(180.0,0.0),egui::pos2(600.0,450.0)).contains_rect(rect),"时间调整按钮应完整显示");
        let pos=rect.center();
        frame(&mut h,vec![egui::Event::PointerMoved(pos),egui::Event::PointerButton{pos,button:egui::PointerButton::Primary,pressed:true,modifiers:Default::default()}]);
        frame(&mut h,vec![egui::Event::PointerButton{pos,button:egui::PointerButton::Primary,pressed:false,modifiers:Default::default()}]);
        assert_eq!(h.app.settings.lyric_delay_ms,expected);
        assert_eq!(crate::model::Settings::load().lyric_delay_ms,expected,"按钮调整后应自动保存");
        text_rect(&frame(&mut h,vec![]),&format!("{:.2} 秒",-expected as f64/1000.0));
    }
}

#[test]
fn island_lyrics_arrive_just_before_vocals_and_finish_fading_before_the_timestamp(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    let mut h=H::new();h.app.settings.island_lyrics=true;
    h.media=crate::smtc::NowPlaying{title:"Track".into(),artist:"Artist".into(),position:9.4,duration:120.0,..Default::default()};
    h.app.set_lyrics("Track","Artist",120.0,crate::lyrics::parse_lrc("[00:05]当前歌词\n[00:10]下一句"));
    for _ in 0..60 {h.frame(vec![]);}
    h.media.playing=true;
    for step in 1..=30 {
        h.media.position=9.4+step as f64/60.0;
        h.frame(vec![]);
        if step==6 {assert_eq!(h.app.island_lyric().map(|(_,line)|line),Some("当前歌词"),"不能提前半秒就跳到下一句");}
    }
    assert_eq!(h.app.island_lyric().map(|(_,line)|line),Some("下一句"),"下一句应在开唱前提前出现");
    let text=h.shapes.iter().find_map(|shape|match &shape.shape {
        egui::Shape::Text(text) if text.galley.text()=="下一句"=>Some(text),_=>None,
    }).expect("开唱前应能看到下一句");
    assert!(text.opacity_factor>0.95,"开唱前应完成淡入，不能等到开唱后才清晰");
    h.media.playing=false;h.frame(vec![]);
    assert_eq!(h.app.island_lyric().map(|(_,line)|line),Some("下一句"),"暂停时不应跳回上一句");
    h.media.position=4.5;h.frame(vec![]);
    assert!(h.app.island_lyric().is_none(),"前奏不能过早显示第一句");
    h.media.position=4.75;h.frame(vec![]);
    assert_eq!(h.app.island_lyric().map(|(_,line)|line),Some("当前歌词"),"第一句也应提前出现");
}

#[test]
fn expanded_and_segmented_lyrics_share_the_early_display_time(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    let mut h=H::new();h.app.settings.island_lyrics=true;
    h.media=crate::smtc::NowPlaying{title:"Track".into(),artist:"Artist".into(),position:9.8,duration:120.0,..Default::default()};
    h.app.set_lyrics("Track","Artist",120.0,crate::lyrics::parse_lrc("[00:05]当前歌词\n[00:10]下一句"));
    h.hover(Pos2::new(WIN_W/2.0,12.0),1.0);
    let next=h.shapes.iter().find_map(|shape|match &shape.shape{egui::Shape::Text(text)if text.galley.text()=="下一句"=>Some(text),_=>None}).unwrap();
    assert_eq!(next.galley.job.sections[0].format.font_id.size,12.0,"展开歌词也应在开唱前高亮下一句");
    h.hover(Pos2::new(WIN_W/2.0,PH+30.0),1.0);
    let line="愿每一个认真生活的人，都能在平凡的日子里找到属于自己的光";
    h.media.position=6.0;h.app.set_lyrics("Track","Artist",120.0,crate::lyrics::parse_lrc(&format!("[00:05]{line}\n[00:15]下一句")));
    for _ in 0..60{h.frame(vec![]);}
    let shown=|h:&H|h.shapes.iter().find_map(|s|match &s.shape{egui::Shape::Text(t)if line.contains(t.galley.text())&&!t.galley.text().is_empty()=>Some(t.galley.text().to_owned()),_=>None}).unwrap();
    let first=shown(&h);
    let boundary=5.0+10.0*first.chars().count()as f64/line.chars().count()as f64;
    h.media.position=boundary-0.2;for _ in 0..30{h.frame(vec![]);}
    assert_ne!(shown(&h),first,"长句的后半段也应使用相同的提前量");
}

#[test]
fn island_lyrics_center_segments_in_a_fixed_height(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    let mut h=H::new();h.app.settings.island_lyrics=true;
    h.media=crate::smtc::NowPlaying{title:"Track".into(),artist:"Artist".into(),position:7.0,duration:120.0,..Default::default()};
    for line in ["晚风","愿每一个认真生活的人，都能在平凡的日子里找到属于自己的光","晚风"] {
        h.app.set_lyrics("Track","Artist",120.0,crate::lyrics::parse_lrc(&format!("[00:05]{line}\n[00:15]下一句")));
        for _ in 0..90 {
            h.frame(vec![]);
            assert!(!h.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Text(t) if t.galley.text()=="09:41")),"切换显示段时应保持歌词布局");
        }
        let text=h.shapes.iter().find_map(|shape|match &shape.shape {
            egui::Shape::Text(text) if !text.galley.text().is_empty() && line.contains(text.galley.text())=>Some(text),_=>None,
        }).expect("岛上应显示当前歌词的一段");
        let center=text.pos.x+text.galley.rect.center().x;
        let offset=if h.app.pet_enabled(){24.0}else{0.0};
        assert!((center-WIN_W/2.0-offset).abs()<1.5,"歌词应在内容区居中");
        assert!(!text.galley.elided,"显示段应完整显示");
        assert_eq!(text.galley.rows.len(),1,"每次只显示一段歌词");
        assert!((h.hit.height()-(crate::app::BAR_H+4.0)).abs()<0.1,"岛应保持固定高度");
        assert!(text.visual_bounding_rect().bottom()<=h.hit.bottom(),"文字应完整显示在岛内");
    }
}

#[test]
fn island_lyric_segments_follow_playback_pause_and_seek(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    let mut h=H::new();h.app.settings.island_lyrics=true;
    let line="愿每一个认真生活的人，都能在平凡的日子里找到属于自己的光";
    h.media=crate::smtc::NowPlaying{title:"Track".into(),artist:"Artist".into(),position:6.0,duration:120.0,..Default::default()};
    h.app.set_lyrics("Track","Artist",120.0,crate::lyrics::parse_lrc(&format!("[00:05]{line}\n[00:15]下一句")));
    let shown=|h:&H|h.shapes.iter().find_map(|s|match &s.shape {egui::Shape::Text(t)=>Some(t.galley.text().to_owned()),_=>None}).unwrap();
    for _ in 0..90 {h.frame(vec![]);}let first=shown(&h);
    h.media.position=14.0;for _ in 0..30 {h.frame(vec![]);}let last=shown(&h);
    assert_ne!(first,last,"播放到后半句时应切换显示段");
    assert_eq!(format!("{first}{last}"),line,"前后显示段应覆盖完整歌词");
    for _ in 0..120 {h.frame(vec![]);}assert_eq!(shown(&h),last,"暂停时显示段应保持不动");
    h.media.position=6.0;for _ in 0..30 {h.frame(vec![]);}assert_eq!(shown(&h),first,"拖回前半句应恢复对应显示段");
    h.media.position=16.0;for _ in 0..30 {h.frame(vec![]);}assert_eq!(shown(&h),"下一句");
}

#[test]
fn island_lyrics_keep_the_layout_during_intro_gaps_loading_and_missing_lyrics(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    let mut h=H::new();h.app.settings.minimal=true;h.app.settings.island_lyrics=true;
    h.media=crate::smtc::NowPlaying{title:"Track".into(),artist:"Artist".into(),position:1.0,duration:120.0,..Default::default()};
    for state in 0..6 {
        h.app.ly=Default::default();
        h.app.ly.key="Track|Artist|120".into();
        h.media.position=if state==1 {7.0}else{1.0};
        match state {
            0=>h.app.ly.lyrics=Some(crate::lyrics::parse_lrc("[00:05]第一句")),
            1=>h.app.ly.lyrics=Some(crate::lyrics::parse_lrc("[00:01]第一句\n[00:05]\n[00:10]下一句")),
            2=>h.app.ly.loading=true,
            3=>h.app.ly.error=Some(crate::lyrics::FetchError::NotFound),
            4=>h.app.ly.error=Some(crate::lyrics::FetchError::Unavailable("503".into())),
            _=>h.app.ly.lyrics=Some(crate::lyrics::from_plain("没有时间戳")),
        }
        for _ in 0..90 {h.frame(vec![]);}
        assert!((h.app.current_width()-crate::app::LYRIC_BAR_W).abs()<2.0,"状态 {state} 应保持歌词岛宽度");
        assert!(h.region.height()>30.0,"状态 {state} 不能变回极简细线");
        assert!(!h.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Text(t) if t.galley.text()=="09:41")),"无歌词时不应切回时钟布局");
    }
}

#[test]
#[ignore="按需导出岛上歌词的实际渲染画面"]
fn island_lyrics_layout_gallery(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    let dest=std::path::PathBuf::from(std::env::var_os("DAOZAI_LYRIC_PREVIEW_DIR").expect("需指定预览目录"));
    std::fs::create_dir_all(&dest).unwrap();
    for (name,raw,position) in [("short",Some("[00:05]晚风"),7.0),("long",Some("[00:05]愿每一个认真生活的人，都能在平凡的日子里找到属于自己的光\n[00:15]下一句"),7.0),("long-later",Some("[00:05]愿每一个认真生活的人，都能在平凡的日子里找到属于自己的光\n[00:15]下一句"),14.0),("intro",Some("[00:20]晚风"),7.0),("missing",None,7.0)] {
        let mut h=H::new();h.app.settings.island_lyrics=true;
        h.media=crate::smtc::NowPlaying{title:"Track".into(),artist:"Artist".into(),position,duration:120.0,..Default::default()};
        if let Some(raw)=raw {h.app.set_lyrics("Track","Artist",120.0,crate::lyrics::parse_lrc(raw));}
        else {h.app.ly.key="Track|Artist|120".into();h.app.ly.error=Some(crate::lyrics::FetchError::NotFound);}
        let mut raster=crate::preview_raster::Raster::new(1280,380);
        for frame in 0..90 {
            let t=frame as f64/60.0;
            let env=Env{t,clock:"09:41:00".into(),today:"2026-10-31".into(),date:"10月31日 周六".into(),media:h.media.clone(),..Default::default()};
            let raw=egui::RawInput{screen_rect:Some(Rect::from_min_size(Pos2::ZERO,Vec2::new(WIN_W,WIN_H))),time:Some(t),
                viewports:std::iter::once((egui::ViewportId::ROOT,egui::ViewportInfo{native_pixels_per_point:Some(2.0),..Default::default()})).collect(),..Default::default()};
            let output=h.ctx.run(raw,|ctx|{
                egui::CentralPanel::default().frame(egui::Frame::new()).show(ctx,|ui|{let win=ui.max_rect();h.app.frame(ui,win,&env);});
            });
            raster.update_textures(&output.textures_delta);
            if frame==89 {
                raster.buf.fill([0.95,0.96,0.98,1.0]);
                raster.draw(&h.ctx.tessellate(output.shapes,output.pixels_per_point),output.pixels_per_point);
                let mut rgba=Vec::new();raster.to_rgba8(&mut rgba);
                let image=image::RgbaImage::from_raw(1280,380,rgba).unwrap();
                image::imageops::crop_imm(&image,300,0,680,170).to_image().save(dest.join(format!("{name}.png"))).unwrap();
            }
            raster.free_textures(&output.textures_delta);
        }
    }
}

#[test]
fn music_lyrics_keep_syncing_when_only_the_collapsed_lyrics_are_enabled() {
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    let mut h=H::new();
    h.app.opts.fetch_lyrics=true;h.app.show_lyrics=false;h.app.settings.island_lyrics=true;
    h.media=crate::smtc::NowPlaying{title:"Track".into(),artist:"Artist".into(),position:7.0,duration:120.0,..Default::default()};
    let (tx,rx)=std::sync::mpsc::channel();
    h.app.ly.key="Track|Artist|120".into();h.app.ly.loading=true;h.app.ly.rx=Some(rx);
    tx.send(Ok(crate::lyrics::parse_lrc("[00:05]当前歌词\n[00:10]下一句"))).unwrap();
    h.frame(vec![]);
    assert!(!h.app.ly.loading);
    assert_eq!(h.app.island_lyric().map(|(_,line)|line),Some("当前歌词"));
    h.media.position=12.0;h.frame(vec![]);
    assert_eq!(h.app.island_lyric().map(|(_,line)|line),Some("下一句"));
}

#[test]
fn music_lyrics_respect_the_switch_and_do_not_show_stale_or_unsynced_lines() {
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    let mut h=H::new();
    h.app.settings.minimal=true;h.app.settings.island_lyrics=true;
    h.media=crate::smtc::NowPlaying{title:"Track".into(),artist:"Artist".into(),position:7.0,duration:120.0,..Default::default()};
    h.app.set_lyrics("Track","Artist",120.0,crate::lyrics::parse_lrc("[00:05]当前歌词"));
    for _ in 0..90 {h.frame(vec![]);}
    assert!(h.region.height()>30.0,"主动开启岛上歌词后应可在收起状态阅读");
    h.app.settings.island_lyrics=false;
    for _ in 0..90 {h.frame(vec![]);}
    assert!(h.app.island_lyric().is_none() && h.region.height()<20.0);
    h.app.settings.island_lyrics=true;
    h.media.title="Another track".into();h.frame(vec![]);
    assert!(h.app.island_lyric().is_none(),"换歌后不能显示上一首的歌词");
    h.app.set_lyrics("Another track","Artist",120.0,crate::lyrics::from_plain("没有时间戳的歌词"));h.frame(vec![]);
    assert!(h.app.island_lyric().is_none(),"不能将纯文本歌词伪装成当前句");
}

#[test]
fn hover_expands_with_overshoot_and_leave_collapses() {
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut h = H::new();
    h.hover(Pos2::new(WIN_W / 2.0, 12.0), 0.1);
    assert!(h.app.progress() < 0.1, "意图缓冲期内不应展开");
    let mut peak: f32 = 0.0;
    for _ in 0..60 {
        h.hover(Pos2::new(WIN_W / 2.0, 12.0), 1.0 / 60.0);
        peak = peak.max(h.app.progress());
    }
    assert!(peak > 1.01 && peak < 1.12, "展开应有轻微过冲, peak={peak}");
    assert!((h.app.progress() - 1.0).abs() < 0.02);
    h.cursor = Some(Pos2::new(WIN_W / 2.0, PH + 30.0));
    for _ in 0..40 {
        h.frame(vec![]);
    }
    assert!(h.app.progress() < 0.02, "移开后应收起");
}

#[test]
fn browser_minimal_preserves_preferences_and_restores_the_previous_mode(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    h.app.settings.pet=true;h.app.settings.minimal=false;h.browser_at_top=Some(true);
    for _ in 0..90 {h.frame(vec![]);}
    assert!(h.hit.height()>20.0,"开关默认关闭时不应自动改变模式");
    h.app.settings.browser_minimal=true;h.app.settings.save();h.app.settings.island_lyrics=true;
    h.media=crate::smtc::NowPlaying{title:"Track".into(),artist:"Artist".into(),position:7.0,duration:120.0,..Default::default()};
    h.app.set_lyrics("Track","Artist",120.0,crate::lyrics::parse_lrc("[00:05]当前歌词"));
    for _ in 0..90 {h.frame(vec![]);}
    assert!(h.hit.height()<10.0,"浏览器顶部受遮挡时应收为细刘海，歌词和吉祥物不能撑大它");
    assert!(h.app.settings.pet && !h.app.settings.minimal,"自动极简不能修改手动模式");
    let saved=crate::model::Settings::load();
    assert!(saved.browser_minimal && saved.pet && !saved.minimal,"应只持久化开关，保留原模式");
    h.browser_at_top=None;for _ in 0..30 {h.frame(vec![]);}
    assert!(h.hit.height()<10.0,"打开岛仔设置或托盘时应保留浏览器避让状态");
    h.browser_at_top=Some(false);for _ in 0..90 {h.frame(vec![]);}
    assert!(h.app.pet_enabled() && h.hit.height()>20.0,"离开浏览器或还原窗口后应恢复");
    h.browser_at_top=Some(true);for _ in 0..90 {h.frame(vec![]);}
    h.app.settings.browser_minimal=false;for _ in 0..90 {h.frame(vec![]);}
    assert!(h.app.pet_enabled() && h.hit.height()>20.0,"关闭开关后应立即恢复");
    h.app.settings.island_lyrics=false;h.app.set_minimal(true);h.browser_at_top=Some(false);for _ in 0..90 {h.frame(vec![]);}
    assert!(h.app.settings.minimal && h.hit.height()<10.0,"离开浏览器不能撤销手动极简");
}

#[test]
fn browser_minimal_closes_the_open_island_but_still_allows_deliberate_hover(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    h.app.settings.browser_minimal=true;
    h.app.show_page(0);h.hover(H::local(300.0,60.0),0.8);
    assert!(h.app.open);
    h.browser_at_top=Some(true);h.frame(vec![]);
    assert!(!h.app.open,"切到浏览器时不能继续保留展开状态遮挡标签栏");
    assert!(!h.hit.contains(Pos2::new(WIN_W/2.0,12.0)),"收起动画期间也应让标签栏接收点击");
    h.hover(Pos2::new(WIN_W/2.0,2.0),0.25);
    assert!(!h.app.open,"短暂碰到刘海不能误触");
    h.hover(Pos2::new(WIN_W/2.0,2.0),0.6);
    assert!(h.app.open,"仍应能主动悬停打开岛");
}

#[test]
fn minimal_hover_excludes_the_surrounding_browser_tabs() {
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    let mut h=H::new();
    h.app.settings.minimal=true;
    for _ in 0..90 {h.frame(vec![]);}
    for cursor in [
        Pos2::new(WIN_W/2.0,crate::app::BAR_H_MIN+1.0),
        Pos2::new(WIN_W/2.0+crate::app::BAR_W_MIN/2.0+2.0,2.0),
        Pos2::new(WIN_W/2.0-crate::app::BAR_W_MIN/2.0-2.0,2.0),
    ] {
        assert!(!h.hit.contains(cursor),"刘海外的标签栏应接收鼠标操作");
        h.hover(cursor,1.0);
        assert!(!h.app.open,"鼠标未触碰极简刘海时不得展开");
    }
}

#[test]
fn minimal_hover_requires_continuous_deliberate_contact() {
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    let mut h=H::new();
    h.app.settings.minimal=true;
    for _ in 0..90 {h.frame(vec![]);}
    let inside=Pos2::new(WIN_W/2.0,2.0);
    h.hover(inside,0.25);
    assert!(!h.app.open,"短暂经过极简刘海不得展开");
    h.hover(Pos2::new(WIN_W/2.0,8.0),0.1);
    h.hover(inside,0.25);
    assert!(!h.app.open,"移出刘海应重新计算停留时间");
    h.hover(inside,0.7);
    assert!(h.app.open,"在刘海内持续停留仍应展开");
    h.click(H::local(398.0,68.0));
    assert_eq!(h.app.page,1,"展开后应能正常操作面板");
}

#[test]
fn minimal_hover_does_not_catch_tabs_during_collapse() {
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    let mut h=H::new();
    h.app.settings.minimal=true;
    h.hover(Pos2::new(WIN_W/2.0,2.0),1.0);
    assert!(h.app.open);
    h.cursor=Some(Pos2::new(WIN_W/2.0,PH+30.0));
    for _ in 0..30 {
        h.frame(vec![]);
        if !h.app.open {break;}
    }
    assert!(!h.app.open);
    assert!(h.app.progress()>0.04,"检查收起动画仍在进行时的命中范围");
    let tab=Pos2::new(WIN_W/2.0,12.0);
    assert!(!h.hit.contains(tab),"收起动画不得继续拦截标签栏");
    h.hover(tab,1.0);
    assert!(!h.app.open);
}

#[test]
fn arrows_and_dots_switch_pages() {
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut h = H::new();
    h.hover(Pos2::new(WIN_W / 2.0, 12.0), 1.0);
    h.click(H::local(398.0, 68.0));
    assert_eq!(h.app.page, 1);
    h.hover(H::local(300.0, 60.0), 0.8);
    h.click(H::local(21.0, 68.0));
    assert_eq!(h.app.page, 0);
    h.hover(H::local(300.0, 60.0), 0.8);
    h.click(H::local(398.0, 68.0));
    h.hover(H::local(300.0, 60.0), 0.8);
    h.click(H::local(398.0, 68.0));
    assert_eq!(h.app.page, 2);
}

#[test]
fn wheel_navigation_reaches_every_page_in_both_directions(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    h.app.shelf.items.clear();h.app.todos.items.clear();
    h.hover(Pos2::new(WIN_W/2.0,12.0),1.0);
    h.hover(H::local(210.0,18.0),0.3);
    for (delta,pages) in [(-40.0,[1,2,3,4,5,6,7,0]),(40.0,[7,6,5,4,3,2,1,0])] {
        for page in pages {
            h.wheel(delta);
            assert_eq!(h.app.page,page,"滚轮翻页不能被空的置物架或待办卡住");
            h.hover(H::local(210.0,18.0),0.25);
        }
    }
}

#[test]
fn wheel_scrolls_lists_only_inside_content_and_leaves_at_the_edge(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    h.app.todos.items=(0..4).map(|i|crate::model::TodoItem{text:format!("任务 {i}"),done_on:None,due:None,reminded:false}).collect();
    h.app.shelf.items=(0..6).map(|i|std::path::PathBuf::from(format!("scroll-test-{i}.txt"))).collect();
    for page in [3,4] {
        h.app.show_page(page);h.hover(H::local(300.0,60.0),1.0);
        h.wheel(-40.0);
        assert_eq!(h.app.page,page,"有更多内容时应先滚动列表");
        assert_eq!(if page==3 {h.app.shelf_first}else{h.app.todo_first},1);
        h.hover(H::local(300.0,60.0),0.25);h.wheel(-40.0);
        assert_eq!(h.app.page,page+1,"列表到底后应能继续翻页");
        h.app.show_page(page);h.hover(H::local(210.0,18.0),0.3);h.wheel(-40.0);
        assert_eq!(h.app.page,page+1,"顶栏滚轮不应被内容列表拦截");
    }
}

#[test]
fn wheel_handles_small_deltas_without_skipping_pages_on_bursts(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    h.hover(Pos2::new(WIN_W/2.0,12.0),1.0);h.hover(H::local(210.0,18.0),0.3);
    for _ in 0..16 {h.wheel(-0.5);}
    assert_eq!(h.app.page,1,"高精度滚轮的小幅输入应能累积成一次翻页");
    h.hover(H::local(210.0,18.0),0.3);
    for _ in 0..5 {h.wheel(-40.0);}
    assert_eq!(h.app.page,2,"一次连续滚动不能瞬间跳过多个页面");
}

#[test]
fn navigation_wraps_between_eight_pages_and_notification_settings_remain_accessible() {
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    h.hover(Pos2::new(WIN_W/2.0,12.0),1.0);h.click(H::local(21.0,68.0));assert_eq!(h.app.page,7);
    h.hover(H::local(300.0,60.0),0.8);h.click(H::local(398.0,68.0));assert_eq!(h.app.page,0);
    h.app.open_settings_category(3);assert!(h.app.settings_open);assert_eq!(h.app.settings_tab,6);
}

#[test]
fn resident_quota_moves_left_and_releases_battery_space(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();h.app.settings.pet=false;
    h.power=(Some(80),true);h.quota_accounts(1);
    for _ in 0..90 {h.frame(vec![]);}
    let before=h.quota_centers()[0];let width=h.app.current_width();assert!(before>WIN_W/2.0);
    h.app.settings.show_battery=false;for _ in 0..90 {h.frame(vec![]);}
    let after=h.quota_centers()[0];assert!(after<WIN_W/2.0,"关闭电量后，额度应补到时间左侧");
    assert!(h.app.current_width()<width-20.0,"关闭组件后应释放实际占位");
    h.app.settings.show_battery=true;h.power=(None,false);for _ in 0..90 {h.frame(vec![]);}
    assert!((h.quota_centers()[0]-after).abs()<0.2,"无电池设备应采用相同补位布局");
    h.click(Pos2::new(after,15.0));assert_eq!(h.app.page(),6,"补位后仍可点击额度圆环");
}

#[test]
fn resident_quotas_coexist_with_music_and_focus(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();h.app.settings.pet=false;h.quota_accounts(3);
    h.media=crate::smtc::NowPlaying{title:"Track".into(),artist:"Artist".into(),playing:true,duration:120.0,..Default::default()};
    h.app.pomo.running=true;h.app.pomo.remaining=600.0;
    for _ in 0..90 {h.frame(vec![]);}
    assert_eq!(h.quota_centers().len(),3,"播放和专注状态不应挤掉常驻额度");
    assert!(h.app.current_width()<PW,"多组件应在单行内排布");
    h.app.settings.minimal=true;for _ in 0..90 {h.frame(vec![]);}
    assert!(h.quota_centers().is_empty());assert!(h.region.height()<20.0,"仍须尊重极简开关");
}

#[test]
fn resident_overflow_stays_clear_of_clock_and_opens_all_accounts(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();h.app.settings.pet=false;h.app.settings.show_battery=false;h.quota_accounts(5);
    for _ in 0..90 {h.frame(vec![]);}
    let text_rect=|label:&str|h.shapes.iter().find_map(|shape|match &shape.shape{
        egui::Shape::Text(text) if text.galley.text()==label=>Some(text.visual_bounding_rect()),_=>None,
    }).expect("常驻文字应可见");
    let more=text_rect("+2");let clock=text_rect("09:41");
    assert!(more.right()+8.0<clock.left(),"更多账号计数不能挤占时钟");
    h.click(more.center());assert_eq!(h.app.page(),6);assert_eq!(h.app.quota.config.accounts.len(),5);
}

#[test]
#[ignore = "按需导出常驻组件排布与补位动画"]
fn resident_layout_gallery(){
    fn capture(h:&mut H,raster:&mut crate::preview_raster::Raster,t:f64,draw:bool)->Vec<u8>{
        let env=Env{t,clock:"09:41:00".into(),today:"2026-10-31".into(),date:"10月31日 周六".into(),battery:h.power.0,charging:h.power.1,media:h.media.clone(),bt_ready:true,..Default::default()};
        let raw=egui::RawInput{screen_rect:Some(Rect::from_min_size(Pos2::ZERO,Vec2::new(WIN_W,WIN_H))),time:Some(t),
            viewports:std::iter::once((egui::ViewportId::ROOT,egui::ViewportInfo{native_pixels_per_point:Some(2.0),..Default::default()})).collect(),..Default::default()};
        let output=h.ctx.run(raw,|ctx|{egui::CentralPanel::default().frame(egui::Frame::new()).show(ctx,|ui|{let win=ui.max_rect();h.app.frame(ui,win,&env);});});
        raster.update_textures(&output.textures_delta);let mut rgba=Vec::new();
        if draw {raster.buf.fill([0.09,0.11,0.18,1.0]);raster.draw(&h.ctx.tessellate(output.shapes,output.pixels_per_point),output.pixels_per_point);raster.to_rgba8(&mut rgba);}
        raster.free_textures(&output.textures_delta);rgba
    }
    fn demo(battery:bool,accounts:usize,music:bool,focus:bool,pet:bool)->H{
        let mut h=H::new();h.app.settings.show_battery=battery;h.app.settings.pet=pet;h.power=(Some(80),true);h.quota_accounts(accounts);
        for (i,a) in h.app.quota.config.accounts.iter().enumerate(){
            if let Some(Ok(snapshot))=h.app.quota.rows.get_mut(&a.id){
                snapshot.windows[0].remaining=82.0-i as f32*11.0;
                if i%2==0 {snapshot.windows.push(crate::quota::Window{label:"5 小时".into(),seconds:Some(18000),remaining:58.0,reset:None,text:None});}
            }
        }
        if music {h.media=crate::smtc::NowPlaying{title:"Track".into(),artist:"Artist".into(),playing:true,duration:120.0,..Default::default()};}
        if focus {h.app.pomo.running=true;h.app.pomo.remaining=1283.0;}
        if pet {h.app.pet.react(crate::extras::PetState::Happy);}
        h
    }
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    let dest=std::path::PathBuf::from(std::env::var_os("DAOZAI_RESIDENT_PREVIEW_DIR").expect("需指定预览目录"));std::fs::create_dir_all(&dest).unwrap();
    let scenes=[(true,1,false,false,false),(false,1,false,false,false),(true,3,false,false,false),(false,3,true,false,false),
        (true,3,true,false,false),(false,3,true,true,false),(false,5,false,false,false),(false,2,false,false,true)];
    for (i,(battery,accounts,music,focus,pet)) in scenes.into_iter().enumerate(){
        let mut h=demo(battery,accounts,music,focus,pet);let mut raster=crate::preview_raster::Raster::new(1280,380);
        for frame in 0..90 {
            let rgba=capture(&mut h,&mut raster,frame as f64/60.0,frame==89);
            if frame==89 {image::save_buffer(dest.join(format!("layout-{i}.png")),&rgba,1280,380,image::ColorType::Rgba8).unwrap();}
        }
    }
    let mut h=demo(true,3,true,false,false);let mut raster=crate::preview_raster::Raster::new(1280,380);
    for frame in 0..240 {
        if frame==60 {h.app.settings.show_battery=false;}if frame==180 {h.app.settings.show_battery=true;}
        let rgba=capture(&mut h,&mut raster,frame as f64/60.0,frame%3==0);
        if frame%3==0 {
            let image=image::RgbaImage::from_raw(1280,380,rgba).unwrap();
            image::imageops::crop_imm(&image,200,0,880,80).to_image().save(dest.join(format!("frame-{frame:03}.png"))).unwrap();
        }
    }
    eprintln!("常驻组件预览：{}",dest.display());
}

#[test]
fn quota_text_stays_inside_the_inner_ring_at_zero_and_full_usage() {
    use crate::quota::{Event,Snapshot,Window};
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    for dual in [true,false] {for remaining in [72.0,0.0,100.0] {
        let mut h=H::new();h.quota_accounts(1);
        let reset=Some((chrono::Local::now()+chrono::Duration::days(400)).timestamp());
        let mut windows=vec![Window{label:"本周".into(),seconds:Some(604800),remaining,reset,text:None}];
        if dual {windows.insert(0,Window{label:"5 小时".into(),seconds:Some(18000),remaining,reset,text:None});}
        h.app.quota_event(Event{id:"preview-0".into(),result:Ok(Snapshot{windows,..Default::default()})});
        h.app.show_page(6);h.hover(H::local(300.0,60.0),1.0);
        let center=H::local(103.0,89.0);
        let inner=h.shapes.iter().filter_map(|shape|match &shape.shape {
            egui::Shape::Circle(circle) if circle.center.distance(center)<0.5 && circle.radius>15.0 && circle.stroke.width>0.0=>Some(circle.radius-circle.stroke.width/2.0),
            egui::Shape::Path(path) if path.points.len()>2 && path.stroke.width>0.0=>{
                let radius=path.points[0].distance(center);
                ((15.0..35.0).contains(&radius) && path.points.iter().all(|point|(point.distance(center)-radius).abs()<0.5)).then_some(radius-path.stroke.width/2.0)
            },
            _=>None,
        }).reduce(f32::min).unwrap_or(35.0);
        for shape in &h.shapes {if let egui::Shape::Text(text)=&shape.shape {
            let rect=text.visual_bounding_rect();
            if rect.center().distance(center)<35.0 {
                for corner in [rect.left_top(),rect.right_top(),rect.left_bottom(),rect.right_bottom()] {
                    assert!(corner.distance(center)+0.5<inner,"{} 的文字不得碰到额度环，双周期={dual}，剩余={remaining}",text.galley.text());
                }
            }
        }}
        let texts:Vec<_>=h.shapes.iter().filter_map(|shape|match &shape.shape {
            egui::Shape::Text(text) if text.visual_bounding_rect().left()>=H::local(157.0,0.0).x && text.visual_bounding_rect().top()>=68.0 && text.visual_bounding_rect().bottom()<=129.0=>Some((text.galley.text().to_owned(),text.visual_bounding_rect())),_=>None,
        }).collect();
        for (i,(a,rect)) in texts.iter().enumerate(){for (b,other) in &texts[i+1..]{assert!(!rect.intersects(*other),"{a} 与 {b} 不得重叠");}}
        let label=if dual {"5 小时"}else{"本周剩余"};
        assert!(h.shapes.iter().any(|shape|matches!(&shape.shape,egui::Shape::Text(text) if text.galley.text()==label)),"周期说明应完整可见");
    }}
}

#[test]
fn quota_accounts_switch_independently_and_hide_with_the_page() {
    use crate::quota::{Account,Provider};
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    for id in ["personal","work"] {h.app.quota.config.accounts.push(Account{id:id.into(),provider:Provider::Codex,alias:id.into(),source:format!("{id}.json").into(),slot:None,enabled:true,pinned:true});}
    h.app.quota.config.selected="personal".into();
    h.app.quota_event(crate::quota::Event{id:"personal".into(),result:crate::quota::parse(Provider::Codex,&serde_json::json!({"rate_limit":{"secondary_window":{"used_percent":20,"limit_window_seconds":604800}}}))});
    h.app.quota_event(crate::quota::Event{id:"work".into(),result:Err("登录已失效".into())});
    assert_eq!(h.app.quota.rows["personal"].as_ref().unwrap().windows[0].remaining,80.0);
    assert!(h.app.quota.rows["work"].is_err());assert_eq!(h.app.quota_bar_indices().len(),2);
    h.hover(Pos2::new(WIN_W/2.0,12.0),1.0);h.app.show_page(6);h.hover(H::local(300.0,60.0),0.7);
    h.click(H::local(202.0,39.0));assert_eq!(h.app.quota.config.selected,"work");
    assert_eq!(crate::quota::Config::load().selected,"work");
    h.app.set_page_visible(6,false);assert!(h.app.quota_bar_indices().is_empty());assert_ne!(h.app.page,6);
    h.app.quota.config.accounts.retain(|a|a.id!="work");h.app.quota_event(crate::quota::Event{id:"work".into(),result:Ok(Default::default())});
    assert!(h.app.quota.rows["work"].is_err(),"移除账号后不得接收晚到的额度数据");
}

#[test]
fn page_visibility_is_saved_and_sets_the_starting_page() {
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    let mut h=H::new();
    assert_eq!(h.app.settings.visible_pages,[true;8]);
    assert!(h.app.settings.show_battery);
    h.app.settings.show_battery=false;
    h.app.set_page_visible(0,false);
    h.app.set_page_visible(3,false);
    h.app.settings.save();
    let settings=crate::model::Settings::load();
    assert!(!settings.show_battery);
    assert_eq!(settings.visible_pages,[false,true,true,false,true,true,true,true]);
    let restarted=App::new(&h.ctx,Options{fetch_lyrics:false,pet_idle:1e9});
    assert_eq!(restarted.page(),1);
    assert_eq!(restarted.settings.visible_pages,settings.visible_pages);
    assert!(!restarted.settings.show_battery);
    std::fs::write(crate::model::config_dir().join("settings.txt"),"visible_pages=000000\n").unwrap();
    assert_eq!(crate::model::Settings::load().visible_pages,[true,false,false,false,false,false,false,false]);
    std::fs::write(crate::model::config_dir().join("settings.txt"),"visible_pages=1000000\n").unwrap();
    assert_eq!(crate::model::Settings::load().visible_pages,[true,false,false,false,false,false,false,true],"旧版页面开关应保留，并开启新增的日程页");
    std::fs::write(crate::model::config_dir().join("settings.txt"),"visible_pages=invalid\n").unwrap();
    assert_eq!(crate::model::Settings::load().visible_pages,[true;8]);
}

#[test]
fn battery_module_hides_power_in_collapsed_and_expanded_headers() {
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    for expanded in [false,true] {
        let mut h=H::new();
        h.app.settings.pet=false;
        h.app.settings.show_battery=false;
        if expanded {h.hover(Pos2::new(WIN_W/2.0,12.0),1.0);} else {h.frame(vec![]);}
        let baseline=h.shapes.clone();
        h.power=(Some(80),true);
        h.t-=1.0/60.0;h.frame(vec![]);
        assert!(h.shapes==baseline,"关闭电量后，本机电池状态不应影响界面布局");
        h.app.settings.show_battery=true;
        h.t-=1.0/60.0;h.frame(vec![]);
        assert!(h.shapes!=baseline,"开启后应显示电量");
        h.power=(None,true);
        h.t-=1.0/60.0;h.frame(vec![]);
        assert!(h.shapes==baseline,"没有电池的设备应自动隐藏电量并释放占位");
    }
}

#[test]
fn battery_module_can_disable_only_the_power_warning() {
    use crate::extras::PetState;
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    let mut h=H::new();
    h.app.settings.show_battery=false;
    h.power=(Some(10),false);h.frame(vec![]);
    assert_eq!(h.app.pet_state(),PetState::Awake);
    h.network=Some(false);h.frame(vec![]);
    assert_eq!(h.app.pet_state(),PetState::Worried);
}

#[test]
fn page_visibility_skips_hidden_pages_in_every_navigation_control() {
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    let mut h=H::new();
    for page in [1,3,4,6,7] {h.app.set_page_visible(page,false);}
    h.hover(Pos2::new(WIN_W/2.0,12.0),1.0);
    for expected in [2,5,0] {
        h.click(H::local(398.0,68.0));
        assert_eq!(h.app.page,expected);
        h.hover(H::local(300.0,60.0),0.7);
    }
    h.click(H::local(PW/2.0+6.0,132.8));
    assert_eq!(h.app.page,2,"底部圆点应仅包含启用的页面");
    h.hover(H::local(300.0,60.0),0.7);
    h.frame(vec![egui::Event::Key{key:egui::Key::ArrowRight,physical_key:None,pressed:true,repeat:false,modifiers:Default::default()}]);
    assert_eq!(h.app.page,5);
    h.frame(vec![egui::Event::Key{key:egui::Key::ArrowRight,physical_key:None,pressed:false,repeat:false,modifiers:Default::default()}]);
    h.hover(H::local(300.0,60.0),0.7);
    h.frame(vec![egui::Event::MouseWheel{unit:egui::MouseWheelUnit::Point,delta:Vec2::new(0.0,40.0),modifiers:Default::default()}]);
    assert_eq!(h.app.page,2,"滚轮应跳过隐藏页面");
    h.click(H::local(21.0,68.0));
    assert_eq!(h.app.page,0);
}

#[test]
fn page_visibility_moves_away_from_hidden_page_and_keeps_one_page() {
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    let mut h=H::new();
    h.app.go(4,1.0);
    h.app.adding=true;
    h.app.set_page_visible(4,false);
    assert_eq!(h.app.page,0);
    assert!(!h.app.is_editing());
    for page in [0,1,3,5,6,7] {h.app.set_page_visible(page,false);}
    assert_eq!(h.app.page,2);
    h.app.set_page_visible(2,false);
    assert!(h.app.settings.visible_pages[2],"不能关闭最后一个页面");
    h.hover(Pos2::new(WIN_W/2.0,12.0),1.0);
    h.click(H::local(398.0,68.0));
    h.click(H::local(21.0,68.0));
    assert_eq!(h.app.page,2);
    h.app.set_page_visible(5,true);
    h.hover(H::local(300.0,60.0),0.7);
    h.click(H::local(398.0,68.0));
    assert_eq!(h.app.page,5,"重新开启的页面应恢复导航");
}

#[test]
fn page_visibility_does_not_reopen_hidden_pages_through_shortcuts_or_drops() {
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    let mut h=H::new();
    h.app.set_page_visible(5,false);
    h.app.show_page(5);
    assert_eq!(h.app.page,0);
    assert!(h.app.settings_open);
    assert_eq!(h.app.settings_tab,7,"隐藏页面的入口应进入对应设置");
    h.app.set_page_visible(3,false);
    let file=crate::model::config_dir().join("shelf-hidden.txt");
    std::fs::create_dir_all(crate::model::config_dir()).unwrap();
    std::fs::write(&file,"test").unwrap();
    h.app.on_drop(vec![file]);
    assert_eq!(h.app.page,0);
    assert!(h.app.shelf.items.is_empty());
}

#[test]
fn mail_page_shows_code_and_copies_on_click() {
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut h = H::new();
    h.app.mail.accounts = vec![crate::mail::Account {
        email: "a@qq.com".into(),
        provider: crate::mail::Provider::Qq,
        enabled: true,
    }];
    h.app.mail_event(crate::mail::Event::Mail {
        item: crate::mail::Item {
            account: "a@qq.com".into(),
            from: "GitHub".into(),
            subject: "verify".into(),
            time: "09:41".into(),
            code: Some("482913".into()),
            summary: "Enter 482913 to finish signing in.".into(),
            links: vec![],
            meeting: None,
            uid: 42,
        },
        new: true,
    });
    assert!(h.app.notification_bell_until > 0.0, "新邮件应点亮铃铛");
    assert!(h.app.alerts.iter().any(|a|a.title=="新邮件"&&a.page==5),"新邮件应进入通知队列，不能只有收起时才可见的铃铛");
    h.app.show_page(5);
    h.hover(Pos2::new(WIN_W / 2.0, 12.0), 0.6);
    h.hover(H::local(300.0, 60.0), 0.4);
    h.click(H::local(60.0, 70.0));
    assert!(
        h.cmds.iter().any(|c| matches!(c, crate::app::Cmd::Copy(t) if t == "482913")),
        "点验证码应复制: {:?}",
        h.cmds
    );
}

#[test]
fn mail_row_opens_summary_and_back_returns() {
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut h = H::new();
    h.app.mail.accounts = vec![crate::mail::Account {
        email: "a@qq.com".into(),
        provider: crate::mail::Provider::Qq,
        enabled: true,
    }];
    h.app.mail_event(crate::mail::Event::Mail {
        item: crate::mail::Item {
            account: "a@qq.com".into(),
            from: "GitHub".into(),
            subject: "verify".into(),
            time: "09:41".into(),
            code: Some("482913".into()),
            summary: "Enter 482913 to finish signing in.".into(),
            links: vec![],
            meeting: None,
            uid: 42,
        },
        new: true,
    });
    h.app.show_page(5);
    h.hover(Pos2::new(WIN_W / 2.0, 12.0), 0.6);
    h.hover(H::local(300.0, 50.0), 0.4);
    h.click(H::local(300.0, 50.0));
    assert_eq!(h.app.mail.open_uid, Some(("a@qq.com".to_string(), 42)), "点邮件行应展开摘要");
    h.hover(H::local(350.0, 48.0), 0.4);
    h.click(H::local(350.0, 48.0));
    assert_eq!(h.app.mail.open_uid, None, "点返回应回到列表");
}

#[test]
fn empty_todo_draft_can_be_cancelled_by_clicking_elsewhere(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    for text in ["","   "]{
        let mut h=H::new();h.app.todos.items=vec![crate::model::TodoItem{text:"保留任务".into(),due:None,done_on:None,reminded:false}];h.app.todos.save();
        h.app.show_page(4);h.hover(H::local(300.0,60.0),1.0);h.click(H::local(70.0,109.0));
        assert!(h.app.adding,"点击添加后应保留空白输入框");
        if !text.is_empty(){h.frame(vec![egui::Event::Text(text.into())]);}
        h.cmds.clear();h.click(H::local(65.0,60.0));
        assert!(!h.app.is_editing(),"空白草稿点击岛内其他位置应退出");
        assert!(h.app.buf.is_empty());assert_eq!(h.app.todos.items.len(),1);assert_eq!(h.app.todos.items[0].text,"保留任务");
        assert_eq!(crate::model::TodoList::load().items.len(),1,"不能保存空白待办");
        h.hover(H::local(65.0,60.0),0.3);
        assert!(!h.cmds.iter().any(|cmd|matches!(cmd,crate::app::Cmd::Focus)),"取消后不能反复激活窗口");
    }
}

#[test]
fn blank_todo_draft_closes_when_window_focus_is_lost(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    h.app.show_page(4);h.hover(H::local(300.0,60.0),1.0);h.click(H::local(70.0,109.0));
    h.cmds.clear();h.frame(vec![egui::Event::WindowFocused(false)]);
    assert!(!h.app.is_editing(),"点击其他软件后空白草稿应自动退出");
    assert!(h.app.todos.items.is_empty());
    assert!(!h.cmds.iter().any(|cmd|matches!(cmd,crate::app::Cmd::Focus)),"失去焦点时不能把窗口再次激活");
}

#[test]
fn todo_input_requests_window_focus_once_and_preserves_typed_drafts(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    h.app.show_page(4);h.hover(H::local(300.0,60.0),1.0);h.cmds.clear();h.click(H::local(70.0,109.0));
    assert_eq!(h.cmds.iter().filter(|cmd|matches!(cmd,crate::app::Cmd::Focus)).count(),1,"新建时只应请求一次窗口焦点");
    h.click(H::local(280.0,46.0));assert!(h.app.adding,"点击输入框不能取消草稿");
    h.frame(vec![egui::Event::Text("尚未保存的内容".into())]);h.cmds.clear();
    h.click(H::local(65.0,60.0));h.frame(vec![egui::Event::WindowFocused(false)]);
    for _ in 0..20{h.frame(vec![]);}
    assert!(h.app.adding);assert_eq!(h.app.buf,"尚未保存的内容");assert!(h.app.todos.items.is_empty());
    assert!(!h.cmds.iter().any(|cmd|matches!(cmd,crate::app::Cmd::Focus)),"已有内容失焦后应保留草稿，不能抢回焦点");
}

#[test]
fn switching_apps_cancels_only_empty_new_todos(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    for text in ["","   ","保留未提交的内容"]{
        let mut h=H::new();h.app.show_page(4);h.hover(H::local(300.0,60.0),1.0);h.click(H::local(70.0,109.0));
        if !text.is_empty(){h.frame(vec![egui::Event::Text(text.into())]);}
        h.cmds.clear();h.app.foreground_changed();h.frame(vec![]);
        assert_eq!(h.app.adding,!text.trim().is_empty());
        assert_eq!(h.app.buf,if text.trim().is_empty(){""}else{text});
        assert!(h.app.todos.items.is_empty());
        assert!(!h.cmds.iter().any(|cmd|matches!(cmd,crate::app::Cmd::Focus)));
    }
}

#[test]
fn drink_adds_water_and_todo_can_be_added() {
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut h = H::new();
    h.hover(Pos2::new(WIN_W / 2.0, 12.0), 1.0);
    h.app.go(2, 1.0);
    h.hover(H::local(300.0, 60.0), 0.8);
    h.click(H::local(70.0, 110.0));
    assert_eq!(h.app.water.drunk_ml, 250);
    assert_eq!(h.app.water.hours[9], 250);

    h.app.go(4, 1.0);
    h.hover(H::local(300.0, 60.0), 0.8);
    h.click(H::local(70.0, 109.0));
    assert!(h.app.adding);
    h.hover(H::local(300.0, 60.0), 0.4);
    h.frame(vec![egui::Event::Text("写周报".into())]);
    h.frame(vec![egui::Event::Key { key: egui::Key::Enter, physical_key: None, pressed: true, repeat: false, modifiers: Default::default() }]);
    h.hover(H::local(300.0, 60.0), 0.1);
    assert_eq!(h.app.todos.items.first().map(|t| t.text.as_str()), Some("写周报"));
    assert!(!h.app.adding);
}

#[test]
fn timer_counts_elapsed_time_after_sleep() {
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut h = H::new();
    h.frame(vec![]);
    h.app.pomo.running = true;
    h.app.pomo.remaining = 120.0;
    h.t += 60.0;
    h.frame(vec![]);
    assert!(h.app.pomo.remaining < 61.0, "挂起恢复后必须补算实际流逝时间");
}

#[test]
fn shelf_remove_keeps_original_file() {
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    let file=crate::model::config_dir().join("保留文件.txt");std::fs::create_dir_all(file.parent().unwrap()).unwrap();std::fs::write(&file,"内容").unwrap();
    h.app.shelf.add(&file).unwrap();h.hover(Pos2::new(WIN_W/2.0,12.0),1.0);h.app.go(3,1.0);h.hover(H::local(300.0,60.0),0.8);
    h.click(H::local(233.0,39.0));assert!(h.app.shelf.items.is_empty());assert!(file.exists());assert!(crate::model::Shelf::load().items.is_empty());
}

#[test]
fn shelf_duplicate_drop_reports_existing_files_and_preserves_the_current_page() {
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    let dir=crate::model::config_dir();std::fs::create_dir_all(&dir).unwrap();
    let files:Vec<_>=(0..7).map(|i|{let path=dir.join(format!("文件-{i}.txt"));std::fs::write(&path,"内容").unwrap();path}).collect();
    h.app.on_drop(files.clone());h.hover(Pos2::new(WIN_W/2.0,12.0),1.0);h.app.shelf_first=0;
    h.app.on_drop(vec![files[0].clone()]);h.hover(H::local(300.0,60.0),0.3);
    assert!(h.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Text(t) if t.galley.text()=="该文件已在置物架中")));
    h.app.on_drop(vec![files[0].clone(),files[1].clone()]);h.hover(H::local(300.0,60.0),0.3);
    assert!(h.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Text(t) if t.galley.text()=="这些文件已在置物架中")));
    assert_eq!(h.app.shelf_first,0,"重复添加不应将列表跳到最后");
    assert_eq!(h.app.shelf.items,files);assert_eq!(crate::model::Shelf::load().items,files);
}

#[test]
fn shelf_drop_distinguishes_new_duplicate_and_missing_files() {
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    let dir=crate::model::config_dir();std::fs::create_dir_all(&dir).unwrap();
    let old=dir.join("已添加.txt");let new=dir.join("新文件.txt");let missing=dir.join("已移动.txt");
    for file in [&old,&new]{std::fs::write(file,"内容").unwrap();}
    h.app.on_drop(vec![old.clone()]);h.app.on_drop(vec![old.clone(),new.clone(),new.clone(),missing.clone()]);
    h.hover(Pos2::new(WIN_W/2.0,12.0),1.0);
    assert!(h.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Text(t) if t.galley.text()=="已添加 1 个文件，2 个已存在，1 个无法添加")));
    assert_eq!(h.app.shelf.items,vec![old.clone(),new]);
    std::fs::remove_file(&old).unwrap();h.app.on_drop(vec![old]);h.hover(H::local(300.0,60.0),0.3);
    assert!(h.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Text(t) if t.galley.text()=="文件不存在或无法访问")));
}
#[test]
fn reminders_are_visible_while_collapsed_and_can_be_disabled(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();h.frame(vec![]);
    h.t+=2701.0;h.frame(vec![]);assert!(h.app.notice.is_some());assert!(h.cmds.iter().any(|c|matches!(c,crate::app::Cmd::Notify(t,_)if t.contains("喝水"))));
    h.app.water.remind=false;h.app.notice=None;h.cmds.clear();h.t+=2800.0;h.frame(vec![]);assert!(h.app.notice.is_none());
}
#[test]
fn water_serving_and_undo_are_consistent(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();h.hover(Pos2::new(WIN_W/2.0,12.0),1.0);h.app.go(2,1.0);h.hover(H::local(300.0,60.0),0.8);
    h.app.settings.serving_ml=350;h.click(H::local(70.0,110.0));assert_eq!(h.app.water.drunk_ml,350);assert_eq!(h.app.water.hours[9],350);
    h.click(H::local(191.0,110.0));assert_eq!(h.app.water.drunk_ml,0);assert_eq!(h.app.water.hours[9],0);
}
#[test]
fn mascot_rest_and_focus_have_distinct_states(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();h.app.opts.pet_idle=45.0;h.frame(vec![]);
    assert_eq!(h.app.pet_state(),crate::extras::PetState::Awake);
    h.t+=50.0;h.frame(vec![]);assert_eq!(h.app.pet_state(),crate::extras::PetState::Dozing);
    h.t+=140.0;h.frame(vec![]);assert_eq!(h.app.pet_state(),crate::extras::PetState::Sleeping);
    h.app.pomo.running=true;assert_eq!(h.app.pet_state(),crate::extras::PetState::Focused);
}

#[test]
fn mascot_stays_beside_the_clock_and_all_resident_components(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    h.app.settings.pet=false;h.power=(Some(80),true);h.quota_accounts(5);
    h.media=crate::smtc::NowPlaying{title:"Track".into(),artist:"Artist".into(),playing:true,duration:120.0,..Default::default()};
    h.app.pomo.running=true;h.app.pomo.remaining=1200.0;
    for _ in 0..120 {h.frame(vec![]);}let width=h.app.current_width();
    h.app.settings.pet=true;for _ in 0..120 {h.frame(vec![]);}
    assert!((h.app.current_width()-width-44.0).abs()<0.1,"岛仔应有独立常驻位置");
    assert_eq!(h.quota_centers().len(),3,"完整排布保留三个额度环");
    assert!(h.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Text(t) if t.galley.text()=="09:41")),"岛仔与时钟共存");
    assert!(h.shapes.iter().filter(|s|matches!(&s.shape,egui::Shape::Path(p) if p.points.len()==13 && (p.stroke.width-3.6).abs()<0.01)).count()>=2,"没有闲置或悬停也应显示眼睛");
}

#[test]
fn mascot_and_album_share_the_single_line_lyric_island(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    h.app.settings.pet=true;h.app.settings.island_lyrics=true;
    h.media=crate::smtc::NowPlaying{title:"Track".into(),artist:"Artist".into(),playing:true,position:7.0,duration:120.0,
        artwork:Some(std::sync::Arc::new(egui::ColorImage::new([32,32],egui::Color32::from_rgb(70,120,220)))),..Default::default()};
    h.app.set_lyrics("Track","Artist",120.0,crate::lyrics::parse_lrc("[00:05]晚风吹过了山岗\n[00:15]下一句"));
    for _ in 0..120 {h.frame(vec![]);}
    assert!((h.app.current_width()-348.0).abs()<0.1,"专辑、岛仔与原有歌词宽度一起保留");
    let texture=h.app.music_artwork.as_ref().unwrap().1.id();
    assert!(h.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Rect(r) if r.brush.as_ref().is_some_and(|b|b.fill_texture_id==texture))),"专辑封面不能被岛仔替代");
    let text=h.shapes.iter().find_map(|s|match &s.shape {egui::Shape::Text(t) if t.galley.text()=="晚风吹过了山岗"=>Some(t),_=>None}).unwrap();
    assert_eq!(text.galley.rows.len(),1,"保留单行歌词");
    assert!((text.pos.x+text.galley.rect.center().x-WIN_W/2.0-24.0).abs()<1.5,"歌词在封面和岛仔右侧居中");
    assert!(h.shapes.iter().filter(|s|matches!(&s.shape,egui::Shape::Path(p) if p.points.len()==13 && (2.79..=3.61).contains(&p.stroke.width))).count()>=2);
}

#[test]
fn singing_microphone_moves_relative_to_the_face_and_disappears_on_pause(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();h.app.settings.pet=true;
    h.media=crate::smtc::NowPlaying{title:"Track".into(),artist:"Artist".into(),playing:true,duration:120.0,..Default::default()};
    for _ in 0..120 {h.frame(vec![]);}
    let head=|shapes:&[egui::epaint::ClippedShape]|shapes.iter().find_map(|s|match &s.shape {
        egui::Shape::Circle(circle) if (circle.radius-2.6).abs()<0.01=>Some(circle.center),_=>None,
    });
    let mut offsets=Vec::new();
    for _ in 0..120 {
        h.frame(vec![]);
        let head=head(&h.shapes).expect("演唱应显示球头麦克风");
        let eyes:Vec<_>=h.shapes.iter().filter_map(|s|match &s.shape {
            egui::Shape::Path(p) if p.points.len()==13 && (p.stroke.width-2.8).abs()<0.01=>Some(p.points.iter().map(|p|p.y).sum::<f32>()/13.0),_=>None,
        }).collect();
        assert_eq!(eyes.len(),2);offsets.push(head.y-(eyes[0]+eyes[1])/2.0);
    }
    if crate::bar_layout::animations_enabled(){
        let range=offsets.iter().copied().fold(f32::NEG_INFINITY,f32::max)-offsets.iter().copied().fold(f32::INFINITY,f32::min);
        assert!(range>0.4,"麦克风需要有独立动态");
    }
    h.media.playing=false;for _ in 0..90 {h.frame(vec![]);}
    assert!(head(&h.shapes).is_none(),"暂停后应收起麦克风");
}

#[test]
fn music_mascot_puts_away_the_microphone_during_notices_and_their_fade_out(){
    use crate::extras::PetState;
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    h.app.settings.pet=true;h.app.settings.island_lyrics=true;
    h.media=crate::smtc::NowPlaying{title:"Track".into(),artist:"Artist".into(),playing:true,position:7.0,duration:120.0,..Default::default()};
    h.app.set_lyrics("Track","Artist",120.0,crate::lyrics::parse_lrc("[00:05]晚风吹过了山岗\n[00:15]下一句"));
    let microphone=|h:&H|h.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Circle(c)if(c.radius-2.6).abs()<0.01));
    for _ in 0..90{h.frame(vec![]);}
    assert_eq!(h.app.pet_state(),PetState::Music);assert!(microphone(&h));
    for (title,body) in [("大写锁定","已开启"),("大写锁定","已关闭"),("键盘输入语言","中文"),("系统音量","音量 60%"),("网络连接","Wi-Fi · 已连接"),("电源已连接","当前电量 80%"),("蓝牙设备","耳机")]{
        h.app.alert(title,body,5,false);
        for _ in 0..60{h.frame(vec![]);assert_eq!(h.app.pet_state(),PetState::Awake,"{title} 出现时应使用普通表情");assert!(!microphone(&h),"{title} 期间不能带着演唱道具");}
        h.t+=2.0;h.frame(vec![]);
        assert!(h.app.notice.is_none());
        assert_eq!(h.app.pet_state(),PetState::Awake,"提示淡出期间也不能提前开始演唱");assert!(!microphone(&h));
        for _ in 0..90{h.frame(vec![]);}
        assert_eq!(h.app.pet_state(),PetState::Music);assert!(microphone(&h),"提示完全结束后应恢复听歌动作");
        assert!(h.media.playing);assert!(h.app.island_lyric().is_some());
    }
    h.app.pet.react(PetState::Satisfied);h.app.alert("大写锁定","已开启",5,false);h.frame(vec![]);
    assert_eq!(h.app.pet_state(),PetState::Satisfied,"已有的喝水反馈应保留优先级");
}

#[test]
fn selecting_permanent_mascot_or_minimal_applies_the_selected_mode(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    h.app.set_minimal(true);for _ in 0..90 {h.frame(vec![]);}
    assert!(h.app.settings.minimal && !h.app.settings.pet && h.region.height()<20.0);
    h.app.set_pet(true);for _ in 0..90 {h.frame(vec![]);}
    assert!(h.app.settings.pet && !h.app.settings.minimal && h.region.height()>30.0);
    let saved=crate::model::Settings::load();assert!(saved.pet && !saved.minimal);
}
#[test]
fn mascot_wakes_before_returning_to_idle(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();h.app.opts.pet_idle=45.0;
    h.frame(vec![]);h.t+=190.0;for _ in 0..60 {h.frame(vec![]);}
    assert_eq!(format!("{:?}",h.app.pet_state()),"Sleeping");
    h.cursor=Some(Pos2::new(550.0,100.0));h.frame(vec![egui::Event::PointerMoved(h.cursor.unwrap())]);
    assert_eq!(format!("{:?}",h.app.pet_state()),"Waking");
    for _ in 0..180 {h.frame(vec![]);}
    assert_eq!(format!("{:?}",h.app.pet_state()),"Awake");
}
#[test]
fn mascot_reacts_to_music_and_respects_focus(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();h.frame(vec![]);
    h.media=crate::smtc::NowPlaying{title:"Track".into(),artist:"Artist".into(),playing:true,duration:120.0,..Default::default()};h.frame(vec![]);
    assert_eq!(format!("{:?}",h.app.pet_state()),"Music");
    h.app.pomo.running=true;h.frame(vec![]);assert_eq!(format!("{:?}",h.app.pet_state()),"Focused");
    h.app.pomo.running=false;h.media.playing=false;h.frame(vec![]);assert_eq!(format!("{:?}",h.app.pet_state()),"Awake");
}
#[test]
fn mascot_keeps_task_celebration_until_panel_closes(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();h.frame(vec![]);
    h.app.todos.items.push(crate::model::TodoItem{text:"完成测试任务".into(),done_on:None,due:None,reminded:false});
    h.hover(Pos2::new(WIN_W/2.0,12.0),1.0);h.app.go(4,1.0);h.hover(H::local(300.0,60.0),1.0);h.click(H::local(250.0,47.0));
    assert!(h.app.todos.items[0].done_on.is_some());
    h.hover(H::local(300.0,60.0),4.0);assert_eq!(format!("{:?}",h.app.pet_state()),"Happy");
    h.cursor=None;for _ in 0..240 {h.frame(vec![]);}
    assert_eq!(format!("{:?}",h.app.pet_state()),"Awake");
}
#[test]
fn mascot_drink_nods_and_focus_completion_celebrates(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();h.frame(vec![]);h.app.drink_water();
    assert_eq!(format!("{:?}",h.app.pet_state()),"Satisfied");
    for _ in 0..240 {h.frame(vec![]);}
    h.app.pomo.running=true;h.app.pomo.remaining=0.01;h.frame(vec![]);
    assert_eq!(format!("{:?}",h.app.pet_state()),"Happy");
}
#[test]
fn mascot_power_and_network_recover_without_stale_warning(){
    use crate::extras::PetState;
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    h.power=(Some(15),false);h.frame(vec![]);assert_eq!(h.app.pet_state(),PetState::Worried);
    h.power.1=true;h.frame(vec![]);assert_eq!(h.app.pet_state(),PetState::Awake);
    h.network=Some(false);h.frame(vec![]);assert_eq!(h.app.pet_state(),PetState::Worried);
    h.network=Some(true);h.frame(vec![]);assert_eq!(h.app.pet_state(),PetState::Awake);
    h.power=(None,false);h.network=None;h.frame(vec![]);assert_eq!(h.app.pet_state(),PetState::Awake);
}
#[test]
fn mascot_system_idle_wakes_once_then_follows_pointer(){
    use crate::extras::PetState;
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();h.app.opts.pet_idle=45.0;
    h.system_idle=Some(200.0);h.frame(vec![]);assert_eq!(h.app.pet_state(),PetState::Sleeping);
    h.system_idle=Some(0.0);h.frame(vec![]);assert_eq!(h.app.pet_state(),PetState::Waking);
    for _ in 0..180 {h.frame(vec![]);}assert_eq!(h.app.pet_state(),PetState::Awake);
    h.cursor=Some(Pos2::new(WIN_W/2.0,50.0));
    for _ in 0..60 {h.frame(vec![]);}assert_eq!(h.app.pet_state(),PetState::Curious);assert!(!h.app.open);
}
#[test]
fn mascot_feedback_plays_beside_notice_and_respects_switches(){
    use crate::extras::PetState;
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();h.frame(vec![]);
    h.app.drink_water();h.app.alert("待办到时间了","测试提醒",4,false);
    for _ in 0..60 {h.frame(vec![]);}assert_eq!(h.app.pet_state(),PetState::Satisfied);
    assert!(h.app.notice.is_some());
    assert!(h.shapes.iter().filter(|s|matches!(&s.shape,egui::Shape::Path(p) if p.points.len()==13 && (p.stroke.width-3.6).abs()<0.01)).count()>=2,"通知期间岛仔仍应可见");
    for _ in 0..240 {h.frame(vec![]);}assert_eq!(h.app.pet_state(),PetState::Awake);
    h.app.notice=None;for _ in 0..240 {h.frame(vec![]);}assert_eq!(h.app.pet_state(),PetState::Awake);
    h.app.drink_water();h.app.settings.minimal=true;h.frame(vec![]);assert!(h.app.pet.reaction.is_none());
    h.app.settings.minimal=false;h.app.settings.pet=false;h.app.drink_water();h.frame(vec![]);assert!(h.app.pet.reaction.is_none());
}

#[test]
#[ignore = "按需导出吉祥物的实际渲染画面"]
fn mascot_expression_gallery(){
    use crate::extras::PetState;
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    let dest=std::env::var_os("DAOZAI_PET_PREVIEW_DIR").map(std::path::PathBuf::from)
        .unwrap_or_else(||std::env::temp_dir().join(format!("daozai-pet-preview-{}",std::process::id())));
    std::fs::create_dir_all(&dest).unwrap();
    let states=[("awake",PetState::Awake),("curious",PetState::Curious),("waking",PetState::Waking),
        ("happy",PetState::Happy),("satisfied",PetState::Satisfied),("music",PetState::Music),
        ("worried",PetState::Worried),("focused",PetState::Focused),("dozing",PetState::Dozing),("sleeping",PetState::Sleeping)];
    for (name,state) in states {
        let mut h=H::new();h.app.pet.react(state);
        let mut raster=crate::preview_raster::Raster::new(1280,380);
        let last=if state==PetState::Waking{42}else{79};
        for frame in 0..=last {
            let t=frame as f64/60.0;
            let env=Env{t,clock:"09:41:00".into(),today:"2026-10-31".into(),date:"10月31日 周六".into(),..Default::default()};
            let raw=egui::RawInput{screen_rect:Some(Rect::from_min_size(Pos2::ZERO,Vec2::new(WIN_W,WIN_H))),time:Some(t),
                viewports:std::iter::once((egui::ViewportId::ROOT,egui::ViewportInfo{native_pixels_per_point:Some(2.0),..Default::default()})).collect(),..Default::default()};
            let output=h.ctx.run(raw,|ctx|{
                egui::CentralPanel::default().frame(egui::Frame::new()).show(ctx,|ui|{
                    let win=ui.max_rect();h.app.frame(ui,win,&env);
                });
            });
            raster.update_textures(&output.textures_delta);
            if frame==last {
                raster.buf.fill([0.09,0.11,0.18,1.0]);
                raster.draw(&h.ctx.tessellate(output.shapes,output.pixels_per_point),output.pixels_per_point);
                let mut rgba=Vec::new();raster.to_rgba8(&mut rgba);
                assert!(rgba.chunks_exact(4).any(|p|p[0]>230&&p[1]>230&&p[2]>230),"表情画面不能空白：{name}");
                image::save_buffer(dest.join(format!("{name}.png")),&rgba,1280,380,image::ColorType::Rgba8).unwrap();
            }
            raster.free_textures(&output.textures_delta);
        }
    }
    eprintln!("表情预览：{}",dest.display());
}
#[test]
fn todo_due_roundtrip_and_fires_only_once(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    h.app.todos.items.push(crate::model::TodoItem{text:"会议 | 准备".into(),done_on:None,due:Some("2026-10-28 09:40".into()),reminded:false});h.app.todos.save();
    let loaded=crate::model::TodoList::load();assert_eq!(loaded.items[0].text,"会议 | 准备");assert_eq!(loaded.items[0].due.as_deref(),Some("2026-10-28 09:40"));
    h.frame(vec![]);assert!(h.app.todos.items[0].reminded);let n=h.cmds.len();h.frame(vec![]);assert_eq!(n,h.cmds.len());assert!(crate::model::TodoList::load().items[0].reminded);
}
#[test]
fn quiet_water_hours_cover_overnight(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();h.app.settings.water_start=22;h.app.settings.water_end=7;
    assert!(h.app.water_hours(23));assert!(h.app.water_hours(2));assert!(!h.app.water_hours(12));
}
#[test]
fn repeated_system_changes_merge_without_losing_history(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();h.frame(vec![]);
    h.app.alert("系统音量","音量 20%",5,false);h.frame(vec![]);
    h.app.alert("系统音量","音量 30%",5,false);h.frame(vec![]);
    assert_eq!(h.app.notice.as_ref().unwrap().body,"音量 30%");assert_eq!(h.app.history.len(),2);assert!(h.app.alerts.is_empty());
}
#[test]
fn windows_notification_echo_does_not_duplicate_own_reminders(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();h.frame(vec![]);
    h.app.alert("喝水时间到了","喝点水吧",2,true);
    h.app.receive_notification("Windows 通知".into(),"test".into(),"喝水时间到了 · 喝点水吧".into());assert_eq!(h.app.history.len(),1);
    h.app.notification_apps.items.push(("mail".into(),"邮件".into(),true));h.app.receive_notification("邮件".into(),"mail".into(),"收到一封新邮件".into());assert_eq!(h.app.alerts.len(),2);assert!(h.app.notification_bell_until>h.app.t);
}

#[test]
fn minimal_takes_precedence_over_mascot(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();h.app.settings.pet=true;h.app.settings.minimal=true;
    for _ in 0..120 {h.frame(vec![]);}assert!(h.region.height()<20.0,"启用极简和吉祥物时必须显示细线");
    h.hover(Pos2::new(WIN_W/2.0,2.0),1.0);assert!(h.app.progress()>0.9,"极简线仍要能悬停展开");
}
#[test]
fn water_settings_are_in_the_island_and_editable(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();h.hover(Pos2::new(WIN_W/2.0,12.0),1.0);
    h.app.go(2,1.0);h.hover(H::local(300.0,60.0),0.8);h.click(H::local(145.0,110.0));assert_eq!(h.app.page,2);assert_eq!(h.app.subpage,Some(2));assert_eq!(h.app.settings_category,1);
    h.hover(H::local(300.0,60.0),0.8);h.click(H::local(193.0,62.0));assert_eq!(h.app.settings.serving_ml,300);
}
#[test]
fn water_notice_buttons_work_without_opening_panel(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();h.frame(vec![]);
    h.app.alert("喝水时间到了","该喝水了",2,false);for _ in 0..60 {h.frame(vec![]);}
    h.click_text("喝一口");assert_eq!(h.app.water.drunk_ml,250);assert!(h.app.notice.is_none());
}
#[test]
fn focus_completion_offers_starting_the_break(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();h.frame(vec![]);h.app.pomo.running=true;h.app.pomo.remaining=1.0;h.t+=2.0;h.frame(vec![]);
    assert!(h.app.pomo.phase==crate::app::Phase::Rest);assert!(!h.app.pomo.running);assert!(h.app.notice.is_some());
    for _ in 0..60 {h.frame(vec![]);}h.click_text("开始");assert!(h.app.pomo.running);
}
#[test]
fn task_editor_saves_and_cancels_inside_island(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();h.hover(Pos2::new(WIN_W/2.0,12.0),1.0);
    h.app.todos.items.push(crate::model::TodoItem{text:"准备会议".into(),done_on:None,due:None,reminded:false});h.app.go(4,1.0);h.hover(H::local(300.0,60.0),0.8);h.app.open_task_dialog(0);
    h.hover(H::local(300.0,60.0),0.2);assert!(h.app.is_editing());h.click(H::local(326.0,110.0));assert!(!h.app.is_editing());assert_eq!(h.app.todos.items[0].text,"准备会议");
    h.app.open_task_dialog(0);h.frame(vec![egui::Event::Key{key:egui::Key::Escape,physical_key:None,pressed:true,repeat:false,modifiers:Default::default()}]);assert!(!h.app.is_editing());
}
#[test]
fn focus_timer_gets_room_in_the_collapsed_island(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();for _ in 0..90 {h.frame(vec![]);}
    let before=h.app.current_width();h.app.pomo.running=true;
    for _ in 0..100 {h.frame(vec![]);}assert!(h.app.current_width()>before+20.0);assert!(h.app.pomo.remaining<1500.0);
}

#[test]
fn switching_to_rest_waits_for_the_start_button(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    h.hover(Pos2::new(WIN_W/2.0,12.0),1.0);h.app.go(1,1.0);h.hover(H::local(260.0,70.0),1.0);
    h.click(H::local(296.0,106.5));
    assert!(h.app.pomo.phase==crate::app::Phase::Rest);
    assert!(!h.app.pomo.running,"切换阶段应等待用户点击开始");
}

#[test]
fn starting_timer_keeps_lyrics_album_and_mascot_with_a_compact_countdown(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    h.app.settings.island_lyrics=true;h.app.settings.pet=true;
    h.media=crate::smtc::NowPlaying{title:"Track".into(),artist:"Artist".into(),playing:true,position:7.0,duration:120.0,
        artwork:Some(std::sync::Arc::new(egui::ColorImage::new([32,32],egui::Color32::from_rgb(70,120,220)))),..Default::default()};
    h.app.set_lyrics("Track","Artist",120.0,crate::lyrics::parse_lrc("[00:05]晚风吹过了山岗\n[00:15]下一句"));
    for _ in 0..120 {h.frame(vec![]);}
    for phase in [crate::app::Phase::Focus,crate::app::Phase::Rest] {
        h.app.pomo.phase=phase;h.app.pomo.remaining=h.app.phase_secs(phase);h.app.pomo.running=true;
        for _ in 0..120 {
            h.frame(vec![]);
            assert_eq!(h.app.island_lyric().map(|(_,line)|line),Some("晚风吹过了山岗"),"计时不能关闭岛上歌词");
            assert!(h.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Text(t) if t.galley.text()=="晚风吹过了山岗" && t.galley.rows.len()==1)));
            assert!(!h.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Text(t) if t.galley.text()=="09:41")));
        }
        let texture=h.app.music_artwork.as_ref().unwrap().1.id();
        assert!(h.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Rect(r) if r.brush.as_ref().is_some_and(|b|b.fill_texture_id==texture))));
        let secs=h.app.pomo.remaining.ceil() as u32;let countdown=format!("{:02}:{:02}",secs/60,secs%60);
        assert!(h.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Text(t) if t.galley.text()==countdown)),"歌词模式中也应显示计时");
        assert!(h.app.current_width()<=PW+0.1);assert!((h.hit.height()-34.0).abs()<0.1);
    }
    h.app.pomo.running=false;for _ in 0..90 {h.frame(vec![]);}
    assert_eq!(h.app.island_lyric().map(|(_,line)|line),Some("晚风吹过了山岗"));
}

#[test]
fn timer_keeps_the_existing_lyric_segments_and_content_bounds(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    let line="愿每一个认真生活的人，都能在平凡的日子里找到属于自己的光";
    for pet in [true,false] {
        let mut h=H::new();h.app.settings.island_lyrics=true;h.app.settings.pet=pet;
        h.media=crate::smtc::NowPlaying{title:"Track".into(),artist:"Artist".into(),position:6.0,duration:120.0,..Default::default()};
        h.app.set_lyrics("Track","Artist",120.0,crate::lyrics::parse_lrc(&format!("[00:05]{line}\n[00:15]下一句")));
        let shown=|h:&H|h.shapes.iter().find_map(|s|match &s.shape {
            egui::Shape::Text(t) if !t.galley.text().is_empty() && line.contains(t.galley.text())=>Some((t.galley.text().to_owned(),t.visual_bounding_rect(),t.galley.rows.len(),t.galley.elided)),_=>None,
        }).unwrap();
        for position in [6.0,14.0] {
            h.media.position=position;h.app.pomo.running=false;
            for _ in 0..90 {h.frame(vec![]);}let expected=shown(&h).0;
            h.app.pomo.running=true;for _ in 0..120 {h.frame(vec![]);}
            let (text,bounds,rows,elided)=shown(&h);assert_eq!(text,expected,"计时不应改变歌词分段");
            assert_eq!(rows,1);assert!(!elided);
            let secs=h.app.pomo.remaining.ceil() as u32;let countdown=format!("{:02}:{:02}",secs/60,secs%60);
            let timer=h.shapes.iter().find_map(|s|match &s.shape {egui::Shape::Text(t) if t.galley.text()==countdown=>Some(t.visual_bounding_rect()),_=>None}).unwrap();
            assert!(bounds.right()+20.0<timer.left(),"歌词与倒计时之间应留出波形和图标的位置");
            assert!(timer.right()<h.hit.right() && bounds.top()>=h.hit.top() && bounds.bottom()<h.hit.bottom());
            assert!((h.app.current_width()-if pet{418.0}else{370.0}).abs()<0.1);
        }
    }
}

#[test]
fn focus_page_selects_stopped_phases_and_starts_only_from_play(){
    use crate::app::Phase;
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    h.app.settings.focus_minutes=30;h.app.settings.rest_minutes=7;
    h.hover(Pos2::new(WIN_W/2.0,12.0),1.0);h.app.go(1,1.0);h.hover(H::local(260.0,70.0),1.0);
    assert!(h.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Text(t) if t.galley.text()=="休息")));
    h.click(H::local(336.0,106.5));
    assert!(h.app.pomo.phase==Phase::Rest && !h.app.pomo.running);
    assert_eq!(h.app.pomo.remaining,420.0,"休息采用设置中的时长");
    h.click(H::local(264.0,106.5));assert!(h.app.pomo.running);
    h.click(H::local(264.0,106.5));assert!(!h.app.pomo.running);
    let paused=h.app.pomo.remaining;for _ in 0..30 {h.frame(vec![]);}assert_eq!(h.app.pomo.remaining,paused);
    h.click(H::local(264.0,106.5));assert!(h.app.pomo.running && h.app.pomo.phase==Phase::Rest);
    h.click(H::local(336.0,106.5));assert!(h.app.pomo.running,"点击当前阶段不重置计时");
    h.click(H::local(192.0,106.5));assert!(h.app.pomo.phase==Phase::Focus && !h.app.pomo.running);
    assert_eq!(h.app.pomo.remaining,1800.0);
    h.click(H::local(264.0,106.5));assert!(h.app.pomo.running);
    h.click(H::local(336.0,106.5));assert!(h.app.pomo.phase==Phase::Rest && !h.app.pomo.running);
    h.click(H::local(264.0,106.5));assert!(h.app.pomo.running);
    h.click(H::local(329.0,50.0));assert!(!h.app.pomo.running && h.app.pomo.phase==Phase::Rest);
    assert_eq!(h.app.pomo.remaining,420.0,"重置应保留休息阶段");
}

#[test]
fn collapsed_focus_uses_tomato_and_rest_uses_a_white_cup(){
    use crate::app::Phase;
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    for phase in [Phase::Focus,Phase::Rest] {
        h.app.pomo.phase=phase;h.app.pomo.remaining=h.app.phase_secs(phase);h.app.pomo.running=true;
        for _ in 0..120 {h.frame(vec![]);}
        let secs=h.app.pomo.remaining.ceil() as u32;let text=format!("{:02}:{:02}",secs/60,secs%60);
        let color=if phase==Phase::Focus{crate::theme::RED_TOP}else{crate::theme::SOFT_GREEN};
        assert!(h.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Text(t) if t.galley.text()==text && t.fallback_color==color)),"常驻倒计时颜色应跟随阶段");
        let tomato=h.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Mesh(m) if m.vertices.iter().any(|v|v.color==crate::theme::RED_TOP)));
        if phase==Phase::Focus {assert!(tomato,"专注时应显示红色番茄水位");}
        else {
            assert!(!tomato,"休息时应替换番茄");
            assert!(h.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Rect(r) if r.fill==egui::Color32::from_rgb(245,242,234) && (7.0..13.0).contains(&r.rect.width()) && (5.0..10.0).contains(&r.rect.height()))),"休息时应显示白瓷茶杯");
        }
        assert!((h.hit.height()-34.0).abs()<0.1,"图标不能撑高常驻岛");
    }
}

#[test]
fn focus_and_rest_completion_prepare_the_configured_next_phase_without_starting(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();h.frame(vec![]);
    h.app.settings.rest_minutes=7;h.app.settings.focus_minutes=30;
    for _ in 0..5 {
        h.app.pomo.phase=crate::app::Phase::Focus;h.app.pomo.running=true;h.app.pomo.remaining=0.01;h.frame(vec![]);
        assert!(h.app.pomo.phase==crate::app::Phase::Rest && !h.app.pomo.running);
        assert_eq!(h.app.pomo.remaining,420.0);assert!(h.app.notice.is_some());
        h.app.pomo.running=true;h.app.pomo.remaining=0.01;h.frame(vec![]);
        assert!(h.app.pomo.phase==crate::app::Phase::Focus && !h.app.pomo.running);
        assert_eq!(h.app.pomo.remaining,1800.0);
    }
}

#[test]
fn rest_settings_keep_the_existing_duration_and_save_the_new_key(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let _h=H::new();
    assert_eq!(crate::model::Settings::default().rest_minutes,5);
    let path=crate::model::config_dir().join("settings.txt");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path,"focus_minutes=30\nshort_minutes=7\nlong_minutes=17\n").unwrap();
    let settings=crate::model::Settings::load();assert_eq!(settings.rest_minutes,7);
    settings.save();let saved=std::fs::read_to_string(&path).unwrap();
    assert!(saved.lines().any(|line|line=="rest_minutes=7"));
    assert!(!saved.contains("short_minutes") && !saved.contains("long_minutes"));
    let reloaded=crate::model::Settings::load();assert_eq!(reloaded.rest_minutes,7);assert_eq!(reloaded.focus_minutes,30);
}

#[test]
fn collapsed_focus_animates_without_music_or_mascot_and_stops_in_minimal_mode(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    h.app.settings.pet=false;h.app.pomo.running=true;
    for _ in 0..120 {h.frame(vec![]);}
    if crate::bar_layout::animations_enabled(){assert!(h.animating,"关闭吉祥物后，常驻番茄的水面仍应轻微波动");}
    h.app.settings.minimal=true;for _ in 0..120 {h.frame(vec![]);}
    assert!(!h.animating,"极简细线隐藏计时图标后，应停止其动画刷新");
}
#[test]
fn app_notifications_show_temporary_bell_in_minimal_mode(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();h.app.settings.minimal=true;h.frame(vec![]);
    h.app.receive_notification("邮件".into(),"mail.app".into(),"私人内容".into());
    assert!(h.app.notification_apps.items[0].2);assert!(h.app.notification_bell_until>h.app.t);assert_eq!(h.app.alerts.len(),1);
    assert!(!std::fs::read_to_string(crate::model::config_dir().join("notification-apps.json")).unwrap().contains("私人内容"));
    for _ in 0..90 {h.frame(vec![]);}assert!(h.region.height()>20.0,"极简模式收到通知时应临时展开");
    h.t+=5.0;for _ in 0..90 {h.frame(vec![]);}assert!(h.region.height()<20.0,"铃铛消失后应恢复极简模式");
    h.app.notification_apps.items[0].2=false;let until=h.app.notification_bell_until;
    h.app.receive_notification("邮件".into(),"mail.app".into(),"私人内容".into());assert_eq!(h.app.notification_bell_until,until);
}

#[test]
fn volume_hud_drag_maps_pointer_to_percentage(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();h.frame(vec![]);
    h.app.alert("系统音量","音量 0%",5,false);
    for _ in 0..60 {h.frame(vec![]);}
    assert!(!h.app.open);
    h.click_notice_badge();
    // 缩短后的系统音量条仍支持点击和拖动。
    let (x0,y)=if h.app.compact_notices(){(WIN_W/2.0+(h.app.current_width()-44.0)/2.0+8.0+crate::app::HUD_TRACK_X,55.0)}else{(WIN_W/2.0-h.app.current_width()/2.0+h.app.pet_space()+crate::app::HUD_TRACK_X,19.0)};
    let press=Pos2::new(x0+52.0,y);
    h.cursor=Some(press);h.frame(vec![egui::Event::PointerMoved(press)]);
    h.frame(vec![egui::Event::PointerButton{pos:press,button:egui::PointerButton::Primary,pressed:true,modifiers:Default::default()}]);
    let to=Pos2::new(x0+105.0,y);
    h.cursor=Some(to);h.frame(vec![egui::Event::PointerMoved(to)]);
    h.frame(vec![egui::Event::PointerButton{pos:to,button:egui::PointerButton::Primary,pressed:false,modifiers:Default::default()}]);
    let last=h.cmds.iter().rev().find_map(|c|match c{crate::app::Cmd::SetVolume(v)=>Some(*v),_=>None});
    assert_eq!(last,Some(70));
}

#[test]
fn bluetooth_connection_soon_after_initial_scan_is_not_suppressed(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    let mut h=H::new();
    h.frame(vec![]);
    h.t=2.0;
    h.bt=vec![crate::bt::Device{id:"shokz".into(),name:"Shokz".into(),battery:Some(100),kind:crate::bt::DeviceKind::Bone}];
    h.frame(vec![]);
    assert_eq!(h.app.notice.as_ref().map(|a|a.title.as_str()),Some("蓝牙设备"));
}

#[test]
fn bluetooth_audio_connection_takes_precedence_over_volume_notice(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    for volume_already_visible in [false,true] {
        let mut h=H::new();h.frame(vec![]);
        h.app.alert("系统音量","音量 80%",5,false);
        if volume_already_visible {h.frame(vec![]);}
        h.bt=vec![crate::bt::Device{id:"audio".into(),name:"Shokz".into(),battery:Some(80),kind:crate::bt::DeviceKind::Bone}];
        h.frame(vec![]);
        assert_eq!(h.app.notice.as_ref().map(|a|a.title.as_str()),Some("蓝牙设备"),"耳机连接应优先于音量提示");
        assert!(!h.app.alerts.iter().any(|a|a.title=="系统音量"),"切换耳机产生的音量提示不应延后补弹");
        h.t+=3.0;h.frame(vec![]);
        h.app.alert("系统音量","音量 60%",5,false);h.frame(vec![]);
        assert_eq!(h.app.notice.as_ref().map(|a|a.body.as_str()),Some("音量 60%"),"连接后手动调节音量应正常提示");
    }
}

#[test]
fn bluetooth_keyboard_connection_keeps_unrelated_volume_notice(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    let mut h=H::new();h.frame(vec![]);
    h.app.alert("系统音量","音量 60%",5,false);h.frame(vec![]);
    h.bt=vec![crate::bt::Device{id:"keyboard".into(),name:"Keyboard".into(),battery:None,kind:crate::bt::DeviceKind::Keyboard}];h.frame(vec![]);
    assert_eq!(h.app.notice.as_ref().map(|a|a.title.as_str()),Some("系统音量"));
    assert!(h.app.alerts.iter().any(|a|a.title=="蓝牙设备"));
}

#[test]
fn bluetooth_notices_track_multiple_devices_independently(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    let mut h=H::new();
    let audio=crate::bt::Device{id:"audio".into(),name:"Shokz".into(),battery:None,kind:crate::bt::DeviceKind::Bone};
    let keyboard=crate::bt::Device{id:"keyboard".into(),name:"QingLong 87".into(),battery:None,kind:crate::bt::DeviceKind::Keyboard};
    h.bt=vec![audio.clone()];h.frame(vec![]);
    h.bt.push(keyboard.clone());h.t=3.0;h.frame(vec![]);
    let notice=h.app.notice.as_ref().unwrap();
    assert_eq!(notice.body,"QingLong 87");
    assert_eq!(notice.bluetooth,Some((crate::bt::DeviceKind::Keyboard,true)));
    h.t=7.0;h.bt=vec![keyboard];h.frame(vec![]);
    let notice=h.app.notice.as_ref().unwrap();
    assert_eq!(notice.body,"Shokz · 已断开");
    assert_eq!(notice.bluetooth,Some((crate::bt::DeviceKind::Bone,false)));
}

#[test]
fn full_notifications_return_when_island_lyrics_are_not_visible(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    for (lyrics,music) in [(false,false),(false,true),(true,false)]{
        let mut h=H::new();h.app.settings.pet=true;h.app.settings.island_lyrics=lyrics;
        if music {h.media=crate::smtc::NowPlaying{title:"Track".into(),artist:"Artist".into(),playing:true,duration:120.0,..Default::default()};}
        for _ in 0..90{h.frame(vec![]);}h.app.alert("大写锁定","已开启",5,false);for _ in 0..60{h.frame(vec![]);}
        for label in ["大写锁定","已开启"]{assert!(h.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Text(t)if t.galley.text()==label&&t.opacity_factor>0.9&&s.clip_rect.contains(t.visual_bounding_rect().center()))),"非岛上歌词状态应直接显示通知文案：{label}");}
        assert!((h.app.current_width()-crate::app::HUD_W-44.0).abs()<1.0,"完整提示保留吉祥物与文案的宽度");
    }
}

#[test]
fn notification_style_follows_the_current_lyric_mode_without_a_new_notice(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    h.media=crate::smtc::NowPlaying{title:"Track".into(),artist:"Artist".into(),playing:true,position:7.0,duration:120.0,..Default::default()};
    h.app.set_lyrics("Track","Artist",120.0,crate::lyrics::parse_lrc("[00:00]晚风吹过了山岗"));
    h.app.alert("大写锁定","已开启",5,false);
    for lyrics in [false,true,false]{
        h.app.settings.island_lyrics=lyrics;for _ in 0..45{h.frame(vec![]);}
        assert_eq!(h.app.compact_notices(),lyrics);
        let has_title=h.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Text(t)if t.galley.text()=="大写锁定"&&t.opacity_factor>0.9));
        assert_eq!(has_title,!lyrics,"当前通知应跟随岛上歌词的显示状态切换样式");
    }
}

#[test]
#[ignore="按需导出普通模式与岛上歌词模式的通知画面"]
fn notice_mode_layout_gallery(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    let dest=std::path::PathBuf::from(std::env::var_os("DAOZAI_NOTICE_MODES_PREVIEW").expect("需指定预览目录"));std::fs::create_dir_all(&dest).unwrap();
    for lyrics in [false,true]{
        let mut h=H::new();h.app.settings.pet=true;h.app.settings.island_lyrics=lyrics;h.app.pomo.running=lyrics;
        h.media=crate::smtc::NowPlaying{title:"Track".into(),artist:"Artist".into(),playing:true,position:7.0,duration:120.0,..Default::default()};
        h.app.set_lyrics("Track","Artist",120.0,crate::lyrics::parse_lrc("[00:00]晚风吹过了山岗"));
        let mut raster=crate::preview_raster::Raster::new(1920,405);
        for frame in 0..120{
            let t=frame as f64/60.0;if frame==30{h.app.alert("大写锁定","已开启",5,false);}
            let env=Env{t,clock:"09:41:00".into(),today:"2026-11-01".into(),date:"11月1日 周日".into(),media:h.media.clone(),..Default::default()};
            let raw=egui::RawInput{screen_rect:Some(Rect::from_min_size(Pos2::ZERO,Vec2::new(crate::app::CANVAS_W,WIN_H))),time:Some(t),viewports:std::iter::once((egui::ViewportId::ROOT,egui::ViewportInfo{native_pixels_per_point:Some(1.5),..Default::default()})).collect(),..Default::default()};
            let output=h.ctx.run(raw,|ctx|{egui::CentralPanel::default().frame(egui::Frame::new()).show(ctx,|ui|{h.app.frame(ui,Rect::from_min_size(Pos2::ZERO,Vec2::new(WIN_W,WIN_H)),&env);});});
            raster.update_textures(&output.textures_delta);
            if frame==119{
                raster.buf.fill([0.94,0.95,0.97,1.0]);raster.draw(&h.ctx.tessellate(output.shapes,output.pixels_per_point),output.pixels_per_point);
                let mut rgba=Vec::new();raster.to_rgba8(&mut rgba);let image=image::RgbaImage::from_raw(1920,405,rgba).unwrap();
                image::imageops::crop_imm(&image,105,0,960,140).to_image().save(dest.join(if lyrics{"岛上歌词模式.png"}else{"普通模式.png"})).unwrap();
            }
            raster.free_textures(&output.textures_delta);
        }
    }
}

#[test]
fn lyric_notifications_extend_right_and_keep_the_lyrics_in_place(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    let text=|h:&H,label:&str|h.shapes.iter().find_map(|s|match &s.shape{egui::Shape::Text(t)if t.galley.text()==label&&t.opacity_factor>0.9&&s.clip_rect.contains(t.visual_bounding_rect().center())=>Some(t.visual_bounding_rect()),_=>None}).unwrap_or_else(||panic!("应保留可见内容：{label}"));
        let mut h=H::new();h.app.settings.pet=true;h.app.settings.island_lyrics=true;h.power=(Some(75),false);
        h.media=crate::smtc::NowPlaying{title:"Track".into(),artist:"Artist".into(),playing:true,position:7.0,duration:120.0,..Default::default()};
        h.app.set_lyrics("Track","Artist",120.0,crate::lyrics::parse_lrc("[00:00]晚风吹过了山岗\n[01:00]下一句"));
        for _ in 0..90{h.frame(vec![]);}
        let label="晚风吹过了山岗";let original=text(&h,label);let right=WIN_W/2.0+h.app.current_width()/2.0;
        h.app.alert("大写锁定","已开启",5,false);
        for _ in 0..60{h.frame(vec![]);assert!(text(&h,label).center().distance(original.center())<0.2,"提示不能挤动或替换原内容");}
        assert!((h.app.current_width()-(right-WIN_W/2.0)*2.0-44.0).abs()<1.0,"通知图标占用 44 点宽度");
        assert!(h.region.right()<right+64.0);assert!(!h.app.open);
        h.t+=3.0;for _ in 0..100{h.frame(vec![]);}
        assert!(text(&h,label).center().distance(original.center())<0.2);
}

#[test]
fn an_open_page_and_lyrics_can_coexist_with_notifications_and_the_bell(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();h.app.settings.island_lyrics=true;
    h.media=crate::smtc::NowPlaying{title:"Track".into(),artist:"Artist".into(),playing:true,position:7.0,duration:120.0,..Default::default()};
    h.app.set_lyrics("Track","Artist",120.0,crate::lyrics::parse_lrc("[00:00]晚风吹过了山岗\n[01:00]下一句"));
    for _ in 0..90{h.frame(vec![]);}
    h.app.notification_apps.items.push(("mail.test".into(),"邮件".into(),true));h.app.receive_notification("邮件".into(),"mail.test".into(),"收到新邮件".into());h.frame(vec![]);
    assert!(h.app.island_lyric().is_some(),"通知铃铛不能关闭岛上歌词");
    h.hover(Pos2::new(WIN_W/2.0,12.0),1.0);h.hover(H::local(300.0,60.0),0.5);assert!(h.app.open);
    h.app.alert("键盘输入语言","中文",5,false);for _ in 0..60{h.frame(vec![]);}
    assert!(h.app.open);
    assert!(h.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Text(t)if t.galley.text()=="Track")),"展开页面保留歌曲信息");
    assert!(!h.app.compact_notices(),"展开页面时不使用岛上歌词的小图标提示");
}

#[test]
#[ignore="按需录制右侧通知与原岛内容同时显示的实际画面"]
fn right_notice_preview(){
    use std::io::Write;
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    let dest=std::path::PathBuf::from(std::env::var_os("DAOZAI_RIGHT_NOTICE_PREVIEW").expect("需指定预览目录"));std::fs::create_dir_all(&dest).unwrap();
    h.app.settings.pet=true;h.app.settings.island_lyrics=true;h.app.pomo.running=true;
    h.media=crate::smtc::NowPlaying{title:"Track".into(),artist:"Artist".into(),playing:true,position:7.0,duration:120.0,..Default::default()};
    h.app.set_lyrics("Track","Artist",120.0,crate::lyrics::parse_lrc("[00:00]晚风吹过了山岗\n[01:00]下一句"));
    let mut video=std::process::Command::new(std::env::var_os("DAOZAI_FFMPEG").unwrap_or_else(||"ffmpeg".into())).args(["-y","-loglevel","error","-f","rawvideo","-pix_fmt","rgba","-s","1920x405","-r","30","-i","-","-vf","crop=1440:210:105:0","-c:v","libx264","-crf","19","-pix_fmt","yuv420p"]).arg(dest.join("右侧通知演示.mp4")).stdin(std::process::Stdio::piped()).spawn().unwrap();
    let mut pipe=video.stdin.take().unwrap();let mut raster=crate::preview_raster::Raster::new(1920,405);
    for frame in 0..420{
        let t=frame as f64/30.0;
        if frame==90{h.app.alert("大写锁定","已开启",5,false);}
        if frame==240{h.app.alert("蓝牙设备","OpenMove by Shokz · 电量 80%",5,false);}
        let env=Env{t,clock:"09:41:00".into(),today:"2026-11-01".into(),date:"11月1日 周日".into(),media:h.media.clone(),..Default::default()};
        let raw=egui::RawInput{screen_rect:Some(Rect::from_min_size(Pos2::ZERO,Vec2::new(crate::app::CANVAS_W,WIN_H))),time:Some(t),viewports:std::iter::once((egui::ViewportId::ROOT,egui::ViewportInfo{native_pixels_per_point:Some(1.5),..Default::default()})).collect(),..Default::default()};
        let output=h.ctx.run(raw,|ctx|{egui::CentralPanel::default().frame(egui::Frame::new()).show(ctx,|ui|{h.app.frame(ui,Rect::from_min_size(Pos2::ZERO,Vec2::new(WIN_W,WIN_H)),&env);});});
        raster.update_textures(&output.textures_delta);raster.buf.fill([0.94,0.95,0.97,1.0]);raster.draw(&h.ctx.tessellate(output.shapes,output.pixels_per_point),output.pixels_per_point);
        let mut rgba=Vec::new();raster.to_rgba8(&mut rgba);pipe.write_all(&rgba).unwrap();
        if let Some(name)=match frame{60=>Some("原岛"),125=>Some("大小写提示"),270=>Some("蓝牙提示"),390=>Some("恢复原岛"),_=>None}{let image=image::RgbaImage::from_raw(1920,405,rgba).unwrap();image::imageops::crop_imm(&image,105,0,1440,210).to_image().save(dest.join(format!("{name}.png"))).unwrap();}
        raster.free_textures(&output.textures_delta);
    }
    drop(pipe);assert!(video.wait().unwrap().success());
}

#[test]
fn hud_expands_smoothly_from_collapsed_width(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    let mut h=H::new();
    for _ in 0..60 { h.frame(vec![]); }
    let before=h.app.current_width();
    h.app.alert("大写锁定","已开启",5,false);
    h.frame(vec![]);
    assert!(h.app.current_width()-before < 30.0, "提示条不能在第一帧突然跳到目标宽度");
}

#[test]
fn bluetooth_notification_keeps_its_full_message_outside_lyric_mode(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    let mut h=H::new();for _ in 0..90 {h.frame(vec![]);}
    h.app.alert("蓝牙设备","OpenMove by Shokz · 电量 80%",5,false);
    for _ in 0..60 {h.frame(vec![]);}
    assert!((h.app.current_width()-crate::app::HUD_W-h.app.pet_space()).abs()<1.0);
    assert!(h.region.height()>60.0,"两行蓝牙提示保持完整高度");
    h.click_notice_badge();
    for label in ["OpenMove by Shokz","已连接","80%"]{assert!(h.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Text(t)if t.galley.text()==label)),"点击后可查看设备信息：{label}");}
}

#[test]
fn hud_repeated_volume_changes_do_not_restart_entrance(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    let mut h=H::new();
    h.frame(vec![]);
    h.app.alert("系统音量","音量 20%",5,false);
    for _ in 0..45 {h.frame(vec![]);}
    let before=h.app.current_width();
    h.app.alert("系统音量","音量 30%",5,false);
    h.frame(vec![]);
    assert!((h.app.current_width()-before).abs()<5.0);
    assert_eq!(h.app.notice.as_ref().unwrap().body,"音量 30%");
}

#[test]
fn hud_shrinks_smoothly_after_expiration(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    let mut h=H::new();
    for _ in 0..90 {h.frame(vec![]);}let collapsed=h.app.current_width();
    h.app.alert("大写锁定","已开启",5,false);
    for _ in 0..60 {h.frame(vec![]);}
    let before=h.app.current_width();
    h.t+=2.0;
    h.frame(vec![]);
    assert!(h.app.notice.is_none());
    assert!(h.app.current_width()>collapsed+5.0);
    assert!(h.app.current_width()<before);
}

#[test]
fn bluetooth_badge_follows_connect_and_disconnect(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    // 首次扫描已连接的设备，记录但不作为新连接提醒。
    h.bt=vec![crate::bt::Device{id:"buds".into(),name:"Buds".into(),battery:Some(80),kind:crate::bt::DeviceKind::Earbuds}];h.frame(vec![]);
    assert!(h.app.notice.is_none());
    assert_eq!(h.app.bt_shown.first().map(|d|d.name.as_str()),Some("Buds"));
    // 之后断开、再连上，各弹一条 HUD 通知。
    h.t=20.0;h.bt.clear();h.frame(vec![]);
    assert_eq!(h.app.notice.as_ref().map(|a|a.body.as_str()),Some("Buds · 已断开"));
    assert!(h.app.bt_shown.is_empty());
    h.t=30.0;h.bt=vec![crate::bt::Device{id:"buds".into(),name:"Buds".into(),battery:Some(80),kind:crate::bt::DeviceKind::Earbuds}];h.frame(vec![]);
    assert_eq!(h.app.notice.as_ref().map(|a|a.body.as_str()),Some("Buds · 电量 80%"));
    // 没有设备时不留残留状态。
    h.t=40.0;h.bt.clear();h.frame(vec![]);
    assert!(h.app.bt_shown.is_empty()&&h.app.notice.as_ref().is_none_or(|a|a.title!="蓝牙设备"||a.body=="Buds · 已断开"));
}

#[test]
fn focus_settings_are_inside_focus_page(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();h.hover(Pos2::new(WIN_W/2.0,12.0),1.0);h.app.go(1,1.0);h.hover(H::local(300.0,60.0),0.8);h.click(H::local(110.0,18.0));assert_eq!(h.app.page,1);assert_eq!(h.app.subpage,Some(1));
}

#[test]
fn calendar_notice_opens_the_calendar_instead_of_mail() {
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    let mut h=H::new();
    assert!(h.app.settings.calendar_auto_sync);
    h.app.settings.calendar_auto_sync=false;
    h.app.settings.save();
    assert!(!crate::model::Settings::load().calendar_auto_sync,"自动同步选择应持久保存");
    let start=chrono::Local::now().timestamp()+600;
    h.app.calendar.config=Some(crate::calendar::Config{provider:crate::calendar::Provider::Feishu,username:"demo".into(),href:None,server:None});
    h.app.calendar_event(crate::calendar::Event::Synced{config:h.app.calendar.config.clone().unwrap(),events:vec![crate::mail::Meeting{ all_day:false, end:None,title:"产品评审".into(),start,at:"10-31 15:00".into(),uid:"calendar-review".into()}],at:"10-31 14:50".into()});
    for _ in 0..60 {h.frame(vec![]);}
    h.click_notice_badge();
    assert!(h.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Text(t) if t.galley.text()=="查看")),"会议提醒按钮使用「查看」");
    assert!(h.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Text(t) if t.galley.text()=="产品评审")),"会议名称应独立显示");
    h.click_text("查看");
    assert!(!h.app.settings_open,"会议提醒不应打开设置页");
    assert_eq!(h.app.page,7,"查看会议必须展开岛上的日程页");
    assert!(h.app.open);
    let date=chrono::DateTime::from_timestamp(start,0).unwrap().with_timezone(&chrono::Local).date_naive();
    assert_eq!(h.app.calendar.view.day,date);
    assert_eq!(h.app.calendar.view.focus.as_deref(),Some(crate::calendar::dedupe_key(&h.app.calendar.events[0])).as_deref());
    assert_ne!(h.app.page,5);
}

#[test]
fn mail_meeting_notice_opens_its_calendar_day() {
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    let mut h=H::new();
    h.app.mail.recent.push(crate::mail::Item{account:"demo@example.com".into(),from:"项目组".into(),subject:"会议邀请".into(),summary:String::new(),links:vec![],code:None,time:"14:50".into(),uid:9,
        meeting:Some(crate::mail::Meeting{ all_day:false, end:None,title:"产品评审".into(),start:chrono::Local::now().timestamp()+600,at:"10-31 15:00".into(),uid:"mail-review".into()})});
    for _ in 0..60 {h.frame(vec![]);}
    h.click_text("查看");
    assert_eq!(h.app.page,7);
    assert!(h.app.open);
    assert_ne!(h.app.page,5);
    let entries=h.app.calendar_entries(h.app.calendar.view.day);
    assert_eq!(entries.len(),1);
    assert!(matches!(entries[0].sources[0],crate::calendar_ui::Source::Mail("demo@example.com")));
    assert!(!h.app.settings_open);
}

#[test]
fn calendar_merges_matching_invites_and_preserves_each_source() {
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    let start=chrono::Local::now().timestamp()+600;
    let event=crate::mail::Meeting{ all_day:false, end:None,title:"产品评审".into(),start,at:String::new(),uid:"shared-review".into()};
    h.app.calendar.events=vec![event.clone()];h.app.calendar.view.events=vec![event.clone()];
    for account in ["work@example.com","personal@example.com"] {
        h.app.mail.recent.push(crate::mail::Item{account:account.into(),from:"项目组".into(),subject:"会议邀请".into(),summary:String::new(),links:vec![],code:None,time:String::new(),uid:9,meeting:Some(event.clone())});
    }
    let day=chrono::DateTime::from_timestamp(start,0).unwrap().with_timezone(&chrono::Local).date_naive();
    for provider in crate::calendar::Provider::ALL {
        h.app.calendar.config=Some(crate::calendar::Config{provider,username:"demo".into(),href:None,server:None});
        let entries=h.app.calendar_entries(day);
        assert_eq!(entries.len(),1,"同一场会议只占一行");
        assert_eq!(entries[0].sources.len(),3,"工作日历及两个邮箱应保留各自来源");
        assert!(matches!(entries[0].sources[0],crate::calendar_ui::Source::Calendar(Some(p)) if p==provider));
    }
    h.app.calendar.events.clear();h.app.calendar.view.events.clear();
    for (i,item) in h.app.mail.recent.iter_mut().enumerate(){let event=item.meeting.as_mut().unwrap();event.uid.clear();event.title=format!("不同会议 {i}");}
    assert_eq!(h.app.calendar_entries(day).len(),2,"缺少会议编号时不能只按相同时间误合并");
}

#[test]
fn calendar_dates_are_readable_and_source_labels_do_not_take_a_second_line() {
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    let day=chrono::NaiveDate::from_ymd_opt(2026,11,15).unwrap();
    let event=crate::mail::Meeting{ all_day:false, end:None,title:"产品评审".into(),start:day.and_hms_opt(14,0,0).unwrap().and_local_timezone(chrono::Local).earliest().unwrap().timestamp(),at:"11-15 14:00".into(),uid:"readable-calendar".into()};
    h.app.calendar.config=Some(crate::calendar::Config{provider:crate::calendar::Provider::Feishu,username:"demo".into(),href:None,server:None});
    h.app.calendar.view.events=vec![event.clone()];
    h.app.mail.recent.push(crate::mail::Item{account:"work@example.com".into(),from:"项目组".into(),subject:"会议邀请".into(),summary:String::new(),links:vec![],code:None,time:String::new(),uid:1,meeting:Some(event.clone())});
    h.app.open_calendar(Some(event));h.hover(H::local(300.0,40.0),1.0);
    let date=h.shapes.iter().find_map(|s|match &s.shape{egui::Shape::Text(t) if t.galley.text()=="15"=>Some(t),_=>None}).expect("月历应显示日期");
    assert!(date.galley.job.sections[0].format.font_id.size>=10.0,"日期数字需要明显放大");
    let date_rect=|label:&str|h.shapes.iter().find_map(|s|match &s.shape{egui::Shape::Text(t) if t.galley.text()==label=>Some(t.visual_bounding_rect()),_=>None}).expect("日期应可见");
    let (ten,eleven,seventeen)=(date_rect("10"),date_rect("11"),date_rect("17"));
    assert!(eleven.left()-ten.right()>=6.0,"相邻日期需要横向留白：{ten:?} {eleven:?}");
    assert!(seventeen.top()-ten.bottom()>=7.0,"相邻周需要纵向留白：{ten:?} {seventeen:?}");
    assert!(!h.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Text(t) if t.galley.text()=="飞书" || t.galley.text().starts_with("邮件 · "))),"来源应使用标题前的 Logo，详细文字保留在悬停提示中");
}

#[test]
fn calendar_first_schedule_has_a_large_time_and_keeps_the_lower_list_scrollable() {
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    let day=chrono::NaiveDate::from_ymd_opt(2026,10,28).unwrap();
    let start=day.and_hms_opt(10,0,0).unwrap().and_local_timezone(chrono::Local).earliest().unwrap().timestamp();
    h.app.calendar.view.day=day;h.app.calendar.view.month=crate::calendar::month_start(day);h.app.calendar.view.loaded=Some(h.app.calendar.view.month);
    h.app.calendar.view.events=(0..7).map(|i|crate::mail::Meeting{ all_day:false,title:format!("会议 {i}"),start:start+i*3600,end:Some(start+i*3600+2700),at:String::new(),uid:format!("layout-{i}")}).collect();
    h.app.open_calendar(None);h.hover(H::local(300.0,40.0),1.0);
    let text=|label:&str|h.shapes.iter().find_map(|s|match &s.shape{egui::Shape::Text(t) if t.galley.text()==label=>Some(t),_=>None}).unwrap();
    assert!(text("10:00").galley.job.sections[0].format.font_id.size>=23.0);
    assert!(text("会议 0").galley.job.sections[0].format.font_id.size>text("会议 1").galley.job.sections[0].format.font_id.size);
    assert!(text("10:45 结束").visual_bounding_rect().bottom()<text("会议 1").visual_bounding_rect().top());
    assert!(text("会议 4").visual_bounding_rect().bottom()<H::local(0.0,205.0).y);
    h.hover(H::local(300.0,194.0),0.25);h.wheel(-40.0);
    assert_eq!(h.app.calendar.view.first,1,"列表底部也应滚动日程");assert_eq!(h.app.page,7);
    assert!(h.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Text(t) if t.galley.text()=="会议 0")),"滚动不能切换大卡片");
    h.click(H::local(260.0,138.0));h.hover(H::local(300.0,194.0),0.2);
    assert!(h.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Text(t) if t.galley.text()=="12:00" && t.galley.job.sections[0].format.font_id.size>=23.0)),"点击才切换大卡片");
    h.wheel(-40.0);
    assert!(h.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Text(t) if t.galley.text()=="12:00" && t.galley.job.sections[0].format.font_id.size>=23.0)),"选中后滚动仍保持详情");
    h.app.open_calendar(Some(h.app.calendar.view.events[6].clone()));h.hover(H::local(300.0,40.0),0.3);
    assert!(h.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Text(t) if t.galley.text()=="会议 6")),"提醒定位的末条日程应完整可见");
}

#[test]
fn calendar_month_navigation_and_today_select_the_right_day() {
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    h.app.open_calendar(None);
    h.app.select_calendar_day(chrono::NaiveDate::from_ymd_opt(2026,12,31).unwrap());
    h.hover(H::local(300.0,60.0),1.0);
    h.click(H::local(129.0,80.0));
    assert_eq!(h.app.calendar.view.month,chrono::NaiveDate::from_ymd_opt(2027,1,1).unwrap());
    let point=h.shapes.iter().find_map(|s|match &s.shape{egui::Shape::Text(t) if t.galley.text()=="15"=>Some(t.visual_bounding_rect().center()),_=>None}).expect("日期应显示在岛上");
    h.click(point);
    assert_eq!(h.app.calendar.view.day,chrono::NaiveDate::from_ymd_opt(2027,1,15).unwrap());
    h.click(H::local(67.0,18.0));
    assert_eq!(h.app.calendar.view.day,chrono::NaiveDate::from_ymd_opt(2026,10,28).unwrap());
    assert_eq!(h.app.page,7);assert!(!h.app.settings_open);
}

#[test]
fn calendar_month_results_preserve_reminders_and_ignore_old_requests() {
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    h.app.calendar.config=Some(crate::calendar::Config{provider:crate::calendar::Provider::Feishu,username:"demo".into(),href:None,server:None});
    let upcoming=crate::mail::Meeting{ all_day:false, end:None,title:"明天会议".into(),start:chrono::Local::now().timestamp()+3600,at:String::new(),uid:"upcoming".into()};
    h.app.calendar.events=vec![upcoming.clone()];
    let month=chrono::NaiveDate::from_ymd_opt(2024,2,1).unwrap();
    h.app.select_calendar_day(month);
    let config=h.app.calendar.config.clone().unwrap();
    h.app.calendar_event(crate::calendar::Event::MonthLoaded{config:config.clone(),month,result:Ok(vec![])});
    assert_eq!(h.app.calendar.events,vec![upcoming]);
    assert_eq!(h.app.calendar.view.loaded,Some(month));
    let next=chrono::NaiveDate::from_ymd_opt(2024,3,1).unwrap();h.app.select_calendar_day(next);
    h.app.calendar_event(crate::calendar::Event::MonthLoaded{config:config.clone(),month,result:Ok(vec![])});
    assert!(h.app.calendar.view.loading,"旧请求不能结束新月份的加载");
    let mut other=config.clone();other.username="another-account".into();
    h.app.calendar_event(crate::calendar::Event::MonthLoaded{config:other,month:next,result:Ok(vec![])});
    assert!(h.app.calendar.view.loading,"其他账号的旧结果应被忽略");
    h.app.calendar_event(crate::calendar::Event::MonthLoaded{config,month:next,result:Err("连接暂不可用".into())});
    assert!(!h.app.calendar.view.loading);assert!(!h.app.calendar.view.error.is_empty());
}

#[test]
fn calendar_island_collapses_and_reopens_on_the_same_page() {
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    h.app.open_calendar(None);h.hover(H::local(300.0,60.0),1.0);
    assert_eq!(h.app.page,7);assert!(h.app.open);
    assert!(h.hit.bottom()>=crate::app::CALENDAR_H,"月历最后一行必须位于可操作区域内");
    h.hover(Pos2::new(600.0,WIN_H+30.0),5.0);assert!(!h.app.open);
    h.hover(Pos2::new(WIN_W/2.0,12.0),1.0);assert!(h.app.open);
    assert_eq!(h.app.page,7);assert!(!h.app.settings_open);
}

#[test]
#[ignore="按需导出岛内日程的实际渲染画面"]
fn calendar_layout_gallery() {
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    let dest=std::path::PathBuf::from(std::env::var_os("DAOZAI_CALENDAR_PREVIEW_DIR").expect("需指定预览目录"));
    std::fs::create_dir_all(&dest).unwrap();
    for mode in ["agenda","dingtalk","wecom","empty","loading","failed","disconnected"] {
        let mut h=H::new();
        let day=chrono::NaiveDate::from_ymd_opt(2026,11,2).unwrap();
        h.app.calendar.view.day=day;h.app.calendar.view.month=crate::calendar::month_start(day);
        h.app.open_calendar(None);
        let provider=match mode{"dingtalk"=>crate::calendar::Provider::DingTalk,"wecom"=>crate::calendar::Provider::WeCom,_=>crate::calendar::Provider::Feishu};
        if mode!="disconnected"{h.app.calendar.config=Some(crate::calendar::Config{provider,username:"demo".into(),href:None,server:None});h.app.calendar.connected=true;}
        h.app.calendar.view.loaded=Some(crate::calendar::month_start(day));
        if ["agenda","dingtalk","wecom"].contains(&mode) {
            for (i,(hour,minute,title)) in [(14,30,"产品方案评审"),(16,0,"下一阶段的设计方案与开发排期对齐"),(17,0,"项目进度同步")].into_iter().enumerate(){
                let at=day.and_hms_opt(hour,minute,0).unwrap().and_local_timezone(chrono::Local).earliest().unwrap();
                h.app.calendar.view.events.push(crate::mail::Meeting{ all_day:false,end:Some(at.timestamp()+2700),title:title.into(),start:at.timestamp(),at:at.format("%m-%d %H:%M").to_string(),uid:format!("sample-{i}")});
            }
            for d in [4,9,12,18,20,25] {let date=chrono::NaiveDate::from_ymd_opt(2026,11,d).unwrap();h.app.calendar.view.events.push(crate::mail::Meeting{ all_day:false, end:None,title:"项目安排".into(),start:date.and_hms_opt(10,0,0).unwrap().and_local_timezone(chrono::Local).earliest().unwrap().timestamp(),at:String::new(),uid:format!("other-{d}")});}
            for (i,(hour,title,account)) in [(15,"客户沟通：合同条款确认","work@example.com"),(16,"交付验收会议","team@example.com")].into_iter().enumerate(){
                let at=day.and_hms_opt(hour,30,0).unwrap().and_local_timezone(chrono::Local).earliest().unwrap();
                h.app.mail.recent.push(crate::mail::Item{account:account.into(),from:"项目组".into(),subject:"会议邀请".into(),summary:String::new(),links:vec![],code:None,time:String::new(),uid:i as u32,
                    meeting:Some(crate::mail::Meeting{ all_day:false, end:None,title:title.into(),start:at.timestamp(),at:at.format("%m-%d %H:%M").to_string(),uid:format!("mail-{i}")})});
            }
            h.app.mail.recent.push(crate::mail::Item{account:"work@example.com".into(),from:"项目组".into(),subject:"会议邀请".into(),summary:String::new(),links:vec![],code:None,time:String::new(),uid:3,meeting:Some(h.app.calendar.view.events[1].clone())});
        }
        if mode=="loading"{h.app.calendar.view.loaded=None;h.app.calendar.view.loading=true;}
        if mode=="failed"{h.app.calendar.view.loaded=None;h.app.calendar.view.error="连不上服务器，检查网络".into();}
        let mut raster=crate::preview_raster::Raster::new(1280,600);
        for f in 0..90 {
            let t=f as f64/60.0;
            let raw=egui::RawInput{screen_rect:Some(Rect::from_min_size(Pos2::ZERO,egui::vec2(WIN_W,WIN_H))),time:Some(t),viewports:std::iter::once((egui::ViewportId::ROOT,egui::ViewportInfo{native_pixels_per_point:Some(2.0),..Default::default()})).collect(),..Default::default()};
            let env=Env{t,today:"2026-11-02".into(),date:"11月2日 周一".into(),clock:"14:15:00".into(),battery:Some(76),charging:true,cursor:Some(H::local(300.0,40.0)),..Default::default()};
            let out=h.ctx.run(raw,|ctx|{egui::CentralPanel::default().frame(egui::Frame::NONE).show(ctx,|ui|{let win=ui.max_rect();h.app.frame(ui,win,&env);});});
            raster.update_textures(&out.textures_delta);
            if f==89 {
                raster.buf.fill([0.035,0.055,0.09,1.0]);raster.draw(&h.ctx.tessellate(out.shapes,out.pixels_per_point),out.pixels_per_point);
                let mut rgba=Vec::new();raster.to_rgba8(&mut rgba);let image=image::RgbaImage::from_raw(1280,600,rgba).unwrap();
                image::imageops::crop_imm(&image,180,0,920,510).to_image().save(dest.join(format!("calendar-{mode}.png"))).unwrap();
            }
            raster.free_textures(&out.textures_delta);
        }
    }
}

#[test]
fn calendar_reminder_fires_once_and_dedupes_with_mail() {
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut h = H::new();
    h.app.mail.accounts=vec![crate::mail::Account{email:"a@qq.com".into(),provider:crate::mail::Provider::Qq,enabled:true}];
    h.app.calendar.config=Some(crate::calendar::Config{provider:crate::calendar::Provider::Feishu,username:"demo".into(),href:None,server:None});
    h.app.settings.calendar_prompt = false;
    let start = chrono::Local::now().timestamp() + 600;
    let at = "10-30 10:00".to_string();
    let meeting = |title: &str| crate::mail::Meeting { all_day:false, end:None, title: title.into(), start, at: at.clone(), uid: "evt-shared-1".into() };
    // 邮件里先来一份，日历再来同一个 UID + DTSTART：只弹一次。
    h.app.mail_event(crate::mail::Event::Mail {
        item: crate::mail::Item {
            account: "a@qq.com".into(),
            from: "腾讯企业邮".into(),
            subject: "周会".into(),
            time: "09:41".into(),
            code: None,
            summary: String::new(),
            links: vec![],
            meeting: Some(meeting("周会")),
            uid: 7,
        },
        new: true,
    });
    h.app.calendar_event(crate::calendar::Event::Synced {
        config:h.app.calendar.config.clone().unwrap(),
        events: vec![meeting("周会（飞书）")],
        at: "10-30 09:00".into(),
    });
    h.frame(vec![]);
    h.frame(vec![]);
    let hits = h.app.history.iter().filter(|a| a.title == "会议提醒").count();
    assert_eq!(hits, 1, "同一场会只提醒一次");
    assert_eq!(h.app.history.iter().find(|a|a.title=="会议提醒").unwrap().page,7,"会议提醒应指向岛内日程页");
    // 再同步一次、再过几帧，也不重复。
    h.app.calendar_event(crate::calendar::Event::Synced { config:h.app.calendar.config.clone().unwrap(),events: vec![meeting("周会（飞书）")], at: "10-30 09:05".into() });
    h.frame(vec![]);
    assert_eq!(h.app.history.iter().filter(|a| a.title == "会议提醒").count(), 1);
}

#[test]
fn calendar_setup_does_not_overlay_music() {
    let _g = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let mut h = H::new();
    h.app.settings.calendar_prompt = true;
    h.app.calendar.config = None;
    h.hover(Pos2::new(WIN_W / 2.0, 12.0), 1.0);
    h.hover(H::local(300.0, 60.0), 0.6);
    h.click(H::local(40.0, 16.0));
    assert!(!h.app.settings_open, "音乐页不应出现日历接入悬浮入口");
    h.app.open_settings_category(3);
    assert!(h.app.settings_open, "仍可从设置访问日程");
    assert_eq!(h.app.settings_tab, 6, "日程位于通知、邮件与日程分组");
}

#[test]
fn language_notice_text_is_vertically_centered() {
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    let mut h=H::new();h.frame(vec![]);
    h.app.alert("键盘输入语言","英语",5,false);
    for _ in 0..60 {h.frame(vec![]);}
    h.click_notice_badge();
    fn find(shapes:&[egui::epaint::ClippedShape])->Option<Rect>{
        shapes.iter().find_map(|s|match &s.shape {
            egui::Shape::Text(text) if text.galley.text()=="英语"=>Some(text.visual_bounding_rect()),
            _=>None,
        })
    }
    let text=find(&h.shapes).expect("输入语言状态应可见");
    assert!((text.center().y-19.0).abs()<1.0,"文字应与图标垂直居中：{text:?}");
}

#[test]
fn todo_notice_view_locates_the_reminded_task() {
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    let mut h=H::new();
    h.app.todos.items=(0..7).map(|i|crate::model::TodoItem{text:format!("任务 {i}"),done_on:None,due:if i==6{Some("2026-10-28 09:40".into())}else{None},reminded:false}).collect();
    for _ in 0..60 {h.frame(vec![]);}
    h.click_text("查看");
    assert_eq!(h.app.page,4);
    assert!(h.app.todo_first>=4,"打开待办页时应能看到收到提醒的任务");
}

#[test]
fn mail_hyperlink_click_requests_the_default_browser() {
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    let mut h=H::new();
    h.app.mail.accounts=vec![crate::mail::Account{email:"a@qq.com".into(),provider:crate::mail::Provider::Qq,enabled:true}];
    h.app.mail.recent=vec![crate::mail::Item{account:"a@qq.com".into(),from:"产品团队".into(),subject:"评审文档".into(),time:"09:41".into(),code:None,summary:"请查看以下文档。".into(),links:vec![crate::mail::Link{label:"查看文档".into(),url:"https://example.com/review?a=1&b=2".into()}],meeting:None,uid:10}];
    h.app.mail.open_uid=Some(("a@qq.com".into(),10));h.app.show_page(5);
    h.hover(H::local(300.0,60.0),1.0);
    let link=h.shapes.iter().find_map(|shape|match &shape.shape{egui::Shape::Text(text)if text.galley.text().contains("查看文档")=>Some(text.visual_bounding_rect()),_=>None}).expect("应展示邮件的超链接");
    h.click(link.center());
    assert!(h.cmds.iter().any(|cmd|matches!(cmd,crate::app::Cmd::OpenUrl(url)if url=="https://example.com/review?a=1&b=2")),"点击后应通过默认浏览器打开完整地址");
}

#[test]
fn new_notification_sways_twice_then_stops_and_restarts_on_the_next_one() {
    for (i,expected_positive) in [true,false,true,false].into_iter().enumerate(){
        let angle=crate::paint::bell_swing(0.95*(i as f64*0.25+0.125));
        assert_eq!(angle>0.0,expected_positive);
    }
    assert_eq!(crate::paint::bell_swing(1.0),0.0);
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();h.frame(vec![]);
    h.app.receive_notification("微信".into(),"wechat".into(),"第一条".into());
    let first=h.app.notification_bell_at;
    for _ in 0..90{h.frame(vec![]);}
    h.app.receive_notification("微信".into(),"wechat".into(),"第二条".into());
    assert!(h.app.notification_bell_at>first+1.0);
    h.frame(vec![]);
    assert!(h.app.notification_bell_until>h.t);
}

#[test]
fn hud_titles_and_states_share_the_icon_center_without_overlapping() {
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    for (title,body,left,right) in [
        ("网络连接","Wi-Fi · 网络已连接","Wi-Fi","网络已连接"),
        ("网络连接","以太网 · 网络已断开","以太网","网络已断开"),
        ("大写锁定","已开启","大写锁定","已开启"),
        ("键盘输入语言","中文","输入语言","中文"),
        ("电源已连接","当前电量 76%","电源已连接","当前电量 76%"),
        ("已切换电池供电","当前电量 76%","电池供电","当前电量 76%"),
    ] {
        let mut h=H::new();h.frame(vec![]);h.app.alert(title,body,5,false);
        for _ in 0..60{h.frame(vec![]);}
        h.click_notice_badge();
        let find=|needle:&str|h.shapes.iter().find_map(|s|match &s.shape{egui::Shape::Text(t)if t.galley.text()==needle=>Some(t.visual_bounding_rect()),_=>None}).unwrap_or_else(||panic!("未显示状态文字：{needle}"));
        let l=find(left);let r=find(right);
        assert!((l.center().y-19.0).abs()<1.0 && (r.center().y-19.0).abs()<1.0,"{title} 应垂直居中");
        assert!(l.right()+8.0<=r.left(),"{title} 的标题和状态应留出间距");
    }
}

#[test]
fn low_power_notice_dismisses_without_opening_the_mail_page() {
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    h.frame(vec![]);h.app.alert("电量偏低","剩余 18% · 请连接电源",5,false);
    for _ in 0..60{h.frame(vec![]);}
    h.click_text("知道了");
    assert!(h.app.notice.is_none());
    assert_eq!(h.app.page,0,"低电量提示只需确认关闭");
}

#[test]
fn code_copy_confirmation_stays_below_the_island_and_does_not_capture_hover() {
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    for detail in [false,true] {
        let mut h=H::new();
        h.app.mail.accounts=vec![crate::mail::Account{email:"a@qq.com".into(),provider:crate::mail::Provider::Qq,enabled:true}];
        h.app.mail.recent=vec![crate::mail::Item{account:"a@qq.com".into(),from:"GitHub".into(),subject:"登录验证码".into(),time:"09:41".into(),code:Some("482913".into()),summary:"请勿向他人透露验证码。".into(),links:vec![],meeting:None,uid:42}];
        if detail {h.app.mail.open_uid=Some(("a@qq.com".into(),42));}
        h.app.show_page(5);h.hover(H::local(300.0,60.0),4.2);
        let copy=H::local(80.0,if detail{212.0}else{70.0});
        h.click(copy);h.hover(copy,0.35);
        let confirmation=|h:&H|h.shapes.iter().find_map(|shape|match &shape.shape {
            egui::Shape::Text(text) if text.galley.text()=="验证码 482913 已复制"=>Some(text.visual_bounding_rect()),_=>None,
        });
        let bounds=confirmation(&h).expect("复制后应显示确认提示");
        assert!(bounds.top()>PH+4.0,"确认提示应在岛下方：{bounds:?}");
        assert!(h.region.contains_rect(bounds),"原生窗口区域应包含提示，避免被裁掉");
        assert!(!h.hit.intersects(bounds),"确认提示不应拦截鼠标或维持岛的展开");
        h.hover(Pos2::new(600.0,180.0),0.8);
        assert!(h.app.progress()<0.02);
        assert!(confirmation(&h).is_some(),"岛收起后确认提示仍应显示到正常结束");
        h.hover(Pos2::new(600.0,180.0),2.0);
        assert!(confirmation(&h).is_none());
        assert!(h.region.bottom()<100.0,"提示结束后应释放额外窗口区域");
    }
}

#[test]
fn adding_a_todo_saves_content_then_offers_an_optional_reminder() {
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    for remind in [false,true] {
        let mut h=H::new();h.app.show_page(4);h.hover(H::local(300.0,60.0),1.0);
        h.click(H::local(70.0,109.0));h.frame(vec![egui::Event::Text("提交产品方案".into())]);
        h.frame(vec![egui::Event::Key{key:egui::Key::Enter,physical_key:None,pressed:true,repeat:false,modifiers:Default::default()}]);
        assert_eq!(h.app.todos.items[0].text,"提交产品方案");
        assert!(h.app.todos.items[0].due.is_none());
        assert!(h.app.task_dialog.is_some(),"保存内容后应进入可选提醒步骤");
        assert_eq!(crate::model::TodoList::load().items[0].text,"提交产品方案");
        h.frame(vec![egui::Event::Key{key:egui::Key::Enter,physical_key:None,pressed:false,repeat:false,modifiers:Default::default()}]);
        if remind {
            let initial=chrono::NaiveDateTime::parse_from_str(&h.app.task_dialog.as_ref().unwrap().picker.due(),"%Y-%m-%d %H:%M").unwrap();
            h.hover(H::local(235.0,77.0),0.4);h.wheel(-40.0);h.picker_click("完成");
            let expected=(initial+chrono::Duration::days(1)).format("%Y-%m-%d %H:%M").to_string();
            assert_eq!(h.app.todos.items[0].due.as_deref(),Some(expected.as_str()));
        }else{h.picker_click("暂不提醒");assert!(h.app.todos.items[0].due.is_none());}
        assert!(h.app.task_dialog.is_none());
        assert!(h.app.todos.items[0].done_on.is_none());
        assert_eq!(crate::model::TodoList::load().items[0].due,h.app.todos.items[0].due);
    }
}

#[test]
fn todo_clock_button_sets_a_reminder_without_completing_the_task() {
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    h.app.todos.items=vec![crate::model::TodoItem{text:"已有任务".into(),done_on:None,due:None,reminded:false}];
    h.app.show_page(4);h.hover(H::local(300.0,60.0),1.0);h.click(H::local(343.0,46.0));
    assert!(h.app.task_dialog.is_some(),"每项待办应有可见的提醒入口");
    assert!(h.app.todos.items[0].done_on.is_none(),"时钟入口不能同时勾选任务");
    h.frame(vec![egui::Event::Key{key:egui::Key::Escape,physical_key:None,pressed:true,repeat:false,modifiers:Default::default()}]);
    assert!(h.app.task_dialog.is_none());assert_eq!(h.app.todos.items[0].text,"已有任务");
}

#[test]
fn mail_details_show_the_receiving_account_when_two_accounts_share_a_uid() {
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    h.app.mail.accounts=vec![crate::mail::Account{email:"first@qq.com".into(),provider:crate::mail::Provider::Qq,enabled:true},crate::mail::Account{email:"work@icloud.com".into(),provider:crate::mail::Provider::ICloud,enabled:true}];
    h.app.mail.recent=["first@qq.com","work@icloud.com"].into_iter().map(|account|crate::mail::Item{account:account.into(),from:"产品团队".into(),subject:"评审安排".into(),time:"09:41".into(),code:None,summary:"请查看会议安排。".into(),links:vec![],meeting:None,uid:42}).collect();
    h.app.show_page(5);h.hover(H::local(300.0,60.0),1.0);
    for (row,account) in ["first@qq.com","work@icloud.com"].into_iter().enumerate(){
        h.hover(H::local(280.0,48.0+row as f32*34.0),0.6);h.click(H::local(280.0,48.0+row as f32*34.0));h.hover(H::local(300.0,90.0),0.1);
        assert_eq!(h.app.mail.open_uid,Some((account.into(),42)));
        assert!(h.shapes.iter().any(|shape|matches!(&shape.shape,egui::Shape::Text(text)if text.galley.text()==format!("收件邮箱：{account}"))),"详情必须明确标出这封邮件所属的收件邮箱");
        h.click(H::local(350.0,48.0));
    }
}

#[test]
fn scrolling_mail_details_keeps_the_current_message_open() {
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    h.app.mail.accounts=vec![crate::mail::Account{email:"work@qq.com".into(),provider:crate::mail::Provider::Qq,enabled:true}];
    h.app.mail.recent=vec![crate::mail::Item{account:"work@qq.com".into(),from:"产品团队".into(),subject:"项目方案".into(),time:"09:41".into(),code:None,summary:"需要滚动阅读的邮件正文。".repeat(20),links:vec![],meeting:None,uid:42}];
    h.app.mail.open_uid=Some(("work@qq.com".into(),42));h.app.show_page(5);h.hover(H::local(270.0,150.0),1.0);
    let body_clip=h.shapes.iter().find_map(|s|match &s.shape{egui::Shape::Text(t) if t.galley.text().starts_with("需要滚动阅读")=>Some(s.clip_rect),_=>None}).expect("正文应显示");
    assert!(body_clip.height()>=80.0,"详情正文应有足够阅读空间，而不是仅能容纳一行");
    h.frame(vec![egui::Event::MouseWheel{unit:egui::MouseWheelUnit::Point,delta:egui::vec2(0.0,-120.0),modifiers:Default::default()}]);
    assert_eq!(h.app.page(),5,"邮件正文滚动不能切换功能页");
    assert_eq!(h.app.mail.open_uid,Some(("work@qq.com".into(),42)));
}

#[test]
fn todo_reminder_picker_is_embedded_in_the_island(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    h.app.todos.items=vec![crate::model::TodoItem{text:"准备会议".into(),done_on:None,due:None,reminded:false}];
    h.app.show_page(4);h.hover(H::local(300.0,60.0),1.0);
    h.ctx.set_embed_viewports(false);
    egui::Context::set_immediate_viewport_renderer(|_,_|panic!("待办提醒必须在岛内选择，不能创建独立窗口"));
    h.click(H::local(343.0,46.0));
    for _ in 0..90 {h.frame(vec![]);}
    assert_eq!(h.app.expanded_height(),PH,"右侧滚轮应保持待办页原来的高度");
    assert!(h.hit.height()<=PH+2.0,"选择时间不能撑大岛体");
    for label in ["准备会议","已完成","取消","完成","暂不提醒"] {
        let (rect,clip)=h.shapes.iter().find_map(|s|match &s.shape {
            egui::Shape::Text(t) if t.galley.text()==label=>Some((t.visual_bounding_rect(),s.clip_rect)),_=>None,
        }).unwrap_or_else(||panic!("岛内缺少选项：{label}"));
        assert!(h.hit.contains_rect(rect),"选项 {label} 应在岛体命中区域内");
        assert!(h.region.contains_rect(rect) && clip.contains_rect(rect),"选项 {label} 不能被窗口或岛体裁剪");
    }
    h.frame(vec![egui::Event::Key{key:egui::Key::Escape,physical_key:None,pressed:true,repeat:false,modifiers:Default::default()}]);
    assert!(h.app.task_dialog.is_none());assert!(h.app.todos.items[0].due.is_none());
}

#[test]
fn reminder_wheels_select_date_hour_and_minute_without_typing(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    let day=chrono::Local::now().date_naive()+chrono::Duration::days(2);let original=format!("{day} 09:00");
    h.app.todos.items=vec![crate::model::TodoItem{text:"准备方案".into(),due:Some(original.clone()),done_on:None,reminded:false}];
    h.app.open_task_reminder(0);
    for x in [235.0,290.0,346.0] {h.hover(H::local(x,77.0),0.4);h.wheel(-40.0);}
    assert_eq!(h.app.todos.items[0].due,Some(original),"调整时不能提前写入提醒");
    h.picker_click("完成");
    let next=day+chrono::Duration::days(1);
    assert_eq!(h.app.todos.items[0].due,Some(format!("{next} 10:01")));
    assert!(h.app.task_dialog.is_none());
}

#[test]
fn reminder_wheels_support_clicks_and_dragging(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    let day=chrono::Local::now().date_naive()+chrono::Duration::days(2);
    h.app.todos.items=vec![crate::model::TodoItem{text:"确认方案".into(),due:Some(format!("{day} 10:30")),done_on:None,reminded:false}];
    h.app.open_task_reminder(0);h.hover(H::local(300.0,60.0),1.0);
    h.click(H::local(346.0,96.0));h.click(H::local(290.0,58.0));
    let start=H::local(235.0,77.0);h.cursor=Some(start);
    h.frame(vec![egui::Event::PointerMoved(start),egui::Event::PointerButton{pos:start,button:egui::PointerButton::Primary,pressed:true,modifiers:Default::default()}]);
    for y in [59.0,41.0]{let point=H::local(235.0,y);h.cursor=Some(point);h.frame(vec![egui::Event::PointerMoved(point)]);}
    let end=H::local(235.0,41.0);
    h.frame(vec![egui::Event::PointerButton{pos:end,button:egui::PointerButton::Primary,pressed:false,modifiers:Default::default()}]);
    h.picker_click("完成");
    assert_eq!(h.app.todos.items[0].due,Some(format!("{} 09:31",day+chrono::Duration::days(2))));
    assert!(h.app.todos.items[0].done_on.is_none());
}

#[test]
fn reminder_wheels_accumulate_small_trackpad_movements(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    let day=chrono::Local::now().date_naive()+chrono::Duration::days(2);
    h.app.todos.items=vec![crate::model::TodoItem{text:"检查进度".into(),due:Some(format!("{day} 10:30")),done_on:None,reminded:false}];
    h.app.open_task_reminder(0);h.hover(H::local(346.0,77.0),1.0);
    for _ in 0..80{h.wheel(-0.5);}
    h.picker_click("完成");
    assert_eq!(h.app.todos.items[0].due,Some(format!("{day} 10:31")));
    assert_eq!(h.app.page(),4,"操作时间滚轮不能触发翻页");
}

#[test]
fn reminder_picker_rejects_past_time_and_cancel_preserves_the_task(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    let old=format!("{} 09:00",chrono::Local::now().date_naive()-chrono::Duration::days(1));
    h.app.todos.items=vec![crate::model::TodoItem{text:"保留任务".into(),due:Some(old.clone()),done_on:None,reminded:true}];
    h.app.open_task_reminder(0);h.picker_click("完成");h.picker_frame(vec![]);
    assert!(h.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Text(t)if t.galley.text()=="请选择晚于当前的时间")));
    assert_eq!(h.app.todos.items[0].due,Some(old.clone()));
    h.picker_click("取消");assert!(h.app.task_dialog.is_none());assert_eq!(h.app.todos.items[0].due,Some(old));
}

#[test]
fn full_mail_reader_keeps_the_entire_body_and_scrolls_independently(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    let body=format!("{}\n最后一段：全文结束", "这是一段需要阅读的完整邮件正文。\n\n".repeat(200));
    let item=crate::mail::Item{account:"work@qq.com".into(),from:"产品团队".into(),subject:"完整方案".into(),time:"09:41".into(),code:None,summary:body.clone(),links:vec![],meeting:None,uid:42};
    h.app.mail.accounts=vec![crate::mail::Account{email:item.account.clone(),provider:crate::mail::Provider::Qq,enabled:true}];
    h.app.mail.recent=vec![item];h.app.mail.open_uid=Some(("work@qq.com".into(),42));h.app.show_page(5);h.hover(H::local(300.0,90.0),1.0);
    h.click(H::local(293.0,48.0));assert_eq!(h.app.mail.reading.as_ref().unwrap().summary,body);
    h.app.mail.recent.clear();
    let render=|h:&mut H,events|{
        h.ctx.set_embed_viewports(true);h.t+=1.0/60.0;
        let out=h.ctx.run(egui::RawInput{screen_rect:Some(Rect::from_min_size(Pos2::ZERO,egui::vec2(700.0,620.0))),time:Some(h.t),events,focused:true,..Default::default()},|ctx|h.app.mail_reader_ui(ctx));h.shapes=out.shapes;
    };
    let top=|h:&H|h.shapes.iter().find_map(|s|match &s.shape{egui::Shape::Text(t)if t.galley.text().ends_with("最后一段：全文结束")=>Some(t.visual_bounding_rect().top()),_=>None}).expect("阅读窗口应包含邮件末尾");
    render(&mut h,vec![]);render(&mut h,vec![]);let before=top(&h);
    render(&mut h,vec![egui::Event::PointerMoved(egui::pos2(350.0,300.0)),egui::Event::MouseWheel{unit:egui::MouseWheelUnit::Point,delta:egui::vec2(0.0,-400.0),modifiers:Default::default()}]);
    for _ in 0..15{render(&mut h,vec![]);}
    assert!(top(&h)<before-100.0,"阅读窗口的滚轮应向下滚动正文");
    assert!(h.app.mail.reading.is_some(),"新邮件列表变化不能打断已经打开的正文");
}

#[test]
#[ignore="按需导出提醒选择器与邮件全文的实际画面"]
fn reminder_and_reader_layout_gallery(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    let dest=std::path::PathBuf::from(std::env::var_os("DAOZAI_READER_PREVIEW_DIR").expect("需指定预览目录"));std::fs::create_dir_all(&dest).unwrap();
    for picker in [true,false]{
        let mut h=H::new();h.ctx.set_embed_viewports(true);
        h.app.todos.items=vec![crate::model::TodoItem{text:"提交产品方案".into(),due:None,done_on:None,reminded:false}];h.app.open_task_reminder(0);
        let body=(1..=12).map(|n|format!("第 {n} 项 · 工作安排\n请确认需求与交付范围，整理设计方案和本周待办，并在评审前同步进度。相关意见请汇总后反馈给产品团队。\n\n")).collect::<String>();
        h.app.mail.reading=Some(crate::mail::Item{account:"work@icloud.com".into(),from:"产品团队".into(),subject:"项目方案与下阶段工作安排".into(),time:"10-31 09:41".into(),code:None,summary:format!("各位好，\n\n以下是项目安排与交付要求，请逐项确认。\n\n{body}最后一段\n请在周五前确认，感谢配合。"),links:vec![crate::mail::Link{label:"查看项目资料".into(),url:"https://example.com/project".into()}],meeting:None,uid:42});
        let (w,height)=if picker{(WIN_W as u32,WIN_H as u32)}else{(700,620)};let ppp=1.5;
        let mut raster=crate::preview_raster::Raster::new((w as f32*ppp)as usize,(height as f32*ppp)as usize);
        let last=if picker{89}else{7};
        for f in 0..=last{
            let raw=egui::RawInput{screen_rect:Some(Rect::from_min_size(Pos2::ZERO,egui::vec2(w as f32,height as f32))),time:Some(f as f64/60.0),viewports:std::iter::once((egui::ViewportId::ROOT,egui::ViewportInfo{native_pixels_per_point:Some(ppp),..Default::default()})).collect(),..Default::default()};
            let out=h.ctx.run(raw,|ctx|if picker{
                let env=Env{t:f as f64/60.0,clock:"09:41:00".into(),today:chrono::Local::now().date_naive().to_string(),date:"周日".into(),..Default::default()};
                egui::CentralPanel::default().frame(egui::Frame::new()).show(ctx,|ui|{let win=ui.max_rect();h.app.frame(ui,win,&env);});
            }else{h.app.mail_reader_ui(ctx)});raster.update_textures(&out.textures_delta);
            if f==last{raster.buf.fill(if picker{[0.07,0.09,0.13,1.0]}else{[0.0,0.0,0.0,1.0]});raster.draw(&h.ctx.tessellate(out.shapes,out.pixels_per_point),out.pixels_per_point);let mut rgba=Vec::new();raster.to_rgba8(&mut rgba);image::save_buffer(dest.join(if picker{"reminder-picker.png"}else{"mail-reader.png"}),&rgba,(w as f32*ppp)as u32,(height as f32*ppp)as u32,image::ColorType::Rgba8).unwrap();}
            raster.free_textures(&out.textures_delta);
        }
    }
}

#[test]
fn remove_icons_are_centered_in_their_buttons_at_each_scale(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    for scale in [1.0,1.25,1.5,2.0]{
        let mut h=H::new();h.ctx.set_pixels_per_point(scale);
        h.app.todos.items=["第一项","第二项","第三项"].into_iter().map(|text|crate::model::TodoItem{text:text.into(),due:None,done_on:None,reminded:false}).collect();
        h.app.show_page(4);h.hover(H::local(300.0,60.0),1.0);
        for i in 0..3usize{
            let button=h.ctx.read_response(egui::Id::new(("card",true,("todo-delete",i)))).unwrap().rect;
            let ink=h.shapes.iter().filter_map(|s|match &s.shape{
                egui::Shape::Text(t) if t.galley.text()=="×" && button.contains(t.visual_bounding_rect().center())=>Some(t.visual_bounding_rect()),
                egui::Shape::LineSegment{points,..} if points.iter().all(|p|button.shrink(2.0).contains(*p)) && (points[0].x-points[1].x).abs()>2.0 && (points[0].y-points[1].y).abs()>2.0=>Some(Rect::from_two_pos(points[0],points[1])),
                _=>None,
            }).fold(Rect::NOTHING,|a,b|a.union(b));
            assert!(ink.is_positive(),"删除按钮应有可见图案");
            assert!(ink.center().distance(button.center())<0.4,"缩放 {scale}：删除图案中心 {:?} 偏离按钮中心 {:?}",ink.center(),button.center());
        }
        h.click(H::local(361.5,68.0));
        assert_eq!(h.app.todos.items.iter().map(|t|t.text.as_str()).collect::<Vec<_>>(),["第一项","第三项"]);
        assert!(h.app.todos.items.iter().all(|t|t.done_on.is_none()),"删除入口不能同时勾选其他待办");
    }
}

#[test]
#[ignore="按需检查全部页面与设置的实际排版"]
fn ui_alignment_layout_gallery(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    let dest=std::path::PathBuf::from(std::env::var_os("DAOZAI_UI_PREVIEW_DIR").expect("需指定预览目录"));std::fs::create_dir_all(&dest).unwrap();
    fn fixture()->H{
        let mut h=H::new();h.app.settings.pet=false;h.app.settings.island_lyrics=false;h.power=(Some(76),false);
        h.media=crate::smtc::NowPlaying{title:"愿每一个认真生活的人，都能在平凡的日子里找到属于自己的光".into(),artist:"The Lanterns · Live Session".into(),duration:240.0,position:65.0,playing:true,..Default::default()};
        h.app.todos.items=["整理会议资料","Review design details","确认项目需求与下一阶段开发排期"].into_iter().map(|text|crate::model::TodoItem{text:text.into(),due:None,done_on:None,reminded:false}).collect();
        h.app.shelf.items=["项目设计方案与交付说明.pdf","design-preview.png","本周工作安排.docx","project-files.zip","notes.md"].into_iter().map(std::path::PathBuf::from).collect();
        h.app.mail.accounts=vec![crate::mail::Account{email:"product-team@example.com".into(),provider:crate::mail::Provider::Qq,enabled:true}];
        h.app.mail.recent=(0..3).map(|uid|crate::mail::Item{account:"product-team@example.com".into(),from:"产品与设计协作团队".into(),subject:"请确认下阶段的设计方案和开发排期，评审前同步最新进度".into(),time:"11-01 09:41".into(),code:if uid==0{Some("123456".into())}else{None},summary:"这是一段邮件正文，需要确认当前需求与后续的产品设计安排。".repeat(20),links:vec![],meeting:None,uid}).collect();
        h.quota_accounts(3);h
    }
    let collapsed=["collapsed","volume","caps","language","power","network","meeting","meeting-dingtalk","meeting-wecom","bt-headphones","bt-keyboard","bt-mouse","bt-gamepad","bt-phone","bt-other","bt-speaker"];
    let scenes=["page-0","page-1","page-2","page-3","page-4","page-5","page-6","page-7","lyrics","todo-empty","shelf-empty","mail-empty","mail-detail","reminder","water-chart","focus-running"];
    for ppp in [1.25,2.0]{for name in scenes.into_iter().chain(collapsed){
        let mut h=fixture();let page=name.strip_prefix("page-").and_then(|p|p.parse::<usize>().ok()).unwrap_or(match name{"lyrics"=>0,"shelf-empty"=>3,"mail-empty"|"mail-detail"=>5,_=>4});
        if name=="todo-empty"{h.app.todos.items.clear();}if name=="shelf-empty"{h.app.shelf.items.clear();}if name=="mail-empty"{h.app.mail.accounts.clear();}
        if !collapsed.contains(&name){h.app.show_page(page);}
        if name=="water-chart"{h.app.show_page(2);h.app.water_view=1;h.app.water.drunk_ml=1250;h.app.water.hours[9]=250;h.app.water.hours[8]=500;h.app.water.hours[7]=500;}
        if name=="focus-running"{h.app.show_page(1);h.app.pomo.running=true;h.app.pomo.remaining=1283.0;}
        if let Some(kind)=name.strip_prefix("bt-"){
            let kind=match kind{"headphones"=>crate::bt::DeviceKind::Headphones,"keyboard"=>crate::bt::DeviceKind::Keyboard,"mouse"=>crate::bt::DeviceKind::Mouse,"gamepad"=>crate::bt::DeviceKind::Gamepad,"phone"=>crate::bt::DeviceKind::Phone,"speaker"=>crate::bt::DeviceKind::Speaker,_=>crate::bt::DeviceKind::Other};
            h.app.bluetooth_alert(&crate::bt::Device{id:"preview".into(),name:"我的蓝牙设备 · 超长设备名称与连接状态".into(),battery:Some(100),kind},true);
        }
        match name{"volume"=>{h.app.volume_ui=Some((64,false));h.app.alert("系统音量","64%",0,false);},"caps"=>h.app.alert("大写锁定","已开启",0,false),"language"=>h.app.alert("键盘输入语言","中文（简体，中国）",0,false),"network"=>h.app.alert("网络连接","Wi-Fi · 网络已连接",0,false),"power"=>h.app.alert("电源已连接","当前电量 76%",0,false),"meeting"=>h.app.meeting_alert(crate::mail::Meeting{ all_day:false,title:"产品方案评审与下阶段开发排期确认".into(),start:chrono::Local::now().timestamp()+600,end:None,at:"11-01 10:00".into(),uid:"preview".into()},"邮件 · product-and-design-team@company-example.com".into(),5),_=>{}}
        if name=="meeting-dingtalk" || name=="meeting-wecom" {
            h.app.meeting_alert(crate::mail::Meeting{all_day:false,title:"项目评审与下阶段安排".into(),start:chrono::Local::now().timestamp()+600,end:None,at:String::new(),uid:name.into()},if name=="meeting-dingtalk"{"钉钉"}else{"企业微信"}.into(),7);
        }
        if name=="lyrics"{h.app.show_lyrics=true;h.app.set_lyrics(&h.media.title,&h.media.artist,240.0,crate::lyrics::parse_lrc("[00:00]慢慢走过这段时光\n[01:00]愿每个认真生活的人都能找到属于自己的光\n[01:10]愿风带你去远方"));}
        if name=="mail-detail"{h.app.mail.open_uid=Some(("product-team@example.com".into(),0));}
        if name=="reminder"{h.app.open_task_reminder(0);}
        let (width,height)=((WIN_W*ppp)as u32,(WIN_H*ppp)as u32);let mut raster=crate::preview_raster::Raster::new(width as usize,height as usize);
        for f in 0..90{
            let t=f as f64/60.0;let raw=egui::RawInput{screen_rect:Some(Rect::from_min_size(Pos2::ZERO,egui::vec2(WIN_W,WIN_H))),time:Some(t),viewports:std::iter::once((egui::ViewportId::ROOT,egui::ViewportInfo{native_pixels_per_point:Some(ppp),..Default::default()})).collect(),..Default::default()};
            let env=Env{t,clock:"09:41:00".into(),today:"2026-11-01".into(),date:"11月1日 周日".into(),cursor:if collapsed.contains(&name){None}else{Some(H::local(150.0,40.0))},media:h.media.clone(),battery:h.power.0,bt_ready:true,..Default::default()};
            let out=h.ctx.run(raw,|ctx|{egui::CentralPanel::default().frame(egui::Frame::NONE).show(ctx,|ui|{let win=ui.max_rect();h.app.frame(ui,win,&env);});});raster.update_textures(&out.textures_delta);
            if f==89{raster.buf.fill([0.07,0.09,0.13,1.0]);raster.draw(&h.ctx.tessellate(out.shapes,out.pixels_per_point),out.pixels_per_point);let mut rgba=Vec::new();raster.to_rgba8(&mut rgba);image::save_buffer(dest.join(format!("{name}-{}.png",(ppp*100.0)as u32)),&rgba,width,height,image::ColorType::Rgba8).unwrap();}
            raster.free_textures(&out.textures_delta);
        }
    }}
    for width in [600.0,690.0]{for tab in 0..12{
        let mut h=fixture();h.app.settings_tab=tab;h.app.open_settings();h.ctx.set_embed_viewports(false);
        let size=egui::vec2(width,if width==600.0{450.0}else{560.0});let ppp=1.25;
        let output=std::sync::Arc::new(Mutex::new(None));let capture=output.clone();
        egui::Context::set_immediate_viewport_renderer(move|ctx,mut viewport|{let id=viewport.ids.this;let raw=egui::RawInput{viewport_id:id,screen_rect:Some(Rect::from_min_size(Pos2::ZERO,size)),viewports:std::iter::once((id,egui::ViewportInfo{native_pixels_per_point:Some(ppp),focused:Some(true),..Default::default()})).collect(),focused:true,..Default::default()};*capture.lock().unwrap()=Some(ctx.run(raw,|ctx|(viewport.viewport_ui_cb)(ctx)));});
        let (w,height)=((size.x*ppp)as u32,(size.y*ppp)as u32);let mut raster=crate::preview_raster::Raster::new(w as usize,height as usize);
        for f in 0..5{let root=h.ctx.run(egui::RawInput::default(),|ctx|h.app.settings_ui(ctx,false));raster.update_textures(&root.textures_delta);let child=output.lock().unwrap().take().unwrap();raster.update_textures(&child.textures_delta);
            if f==4{raster.buf.fill([0.0,0.0,0.0,1.0]);raster.draw(&h.ctx.tessellate(child.shapes,child.pixels_per_point),child.pixels_per_point);let mut rgba=Vec::new();raster.to_rgba8(&mut rgba);image::save_buffer(dest.join(format!("settings-{tab}-{}.png",width as u32)),&rgba,w,height,image::ColorType::Rgba8).unwrap();}
            raster.free_textures(&root.textures_delta);raster.free_textures(&child.textures_delta);
        }
    }}
}

#[test]
fn meeting_notice_prioritizes_the_start_time_and_keeps_actions_aligned(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    for ppp in [1.0,1.25,1.5,2.0]{
        let mut h=H::new();h.ctx.set_pixels_per_point(ppp);h.app.settings.pet=false;
        let start=chrono::Local::now()+chrono::Duration::minutes(10);
        h.app.meeting_alert(crate::mail::Meeting{ all_day:false,title:"产品评审".into(),start:start.timestamp(),end:None,at:start.format("%m-%d %H:%M").to_string(),uid:"meeting-layout".into()},"邮件 · product-and-design-team@company-example.com".into(),7);
        for _ in 0..90{h.frame(vec![]);}
        h.click_notice_badge();
        assert!(!h.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Text(t) if t.galley.text().starts_with("邮件 · "))),"软件名称仅用于识别图标，不占用提醒正文");
        let clock=h.shapes.iter().find_map(|s|match &s.shape{egui::Shape::Text(t) if t.galley.text()==start.format("%H:%M").to_string()=>Some((t.visual_bounding_rect(),s.clip_rect,t.pos.x,t.galley.job.sections[0].format.font_id.size)),_=>None}).unwrap();
        let countdown=h.shapes.iter().find_map(|s|match &s.shape{egui::Shape::Text(t) if t.galley.text().ends_with("分钟后")=>Some((t.visual_bounding_rect(),t.galley.job.sections[0].format.color)),_=>None}).unwrap();
        assert!(clock.0.right()+12.0<=countdown.0.left(),"缩放 {ppp}：开始时刻与倒计时应保持间距");
        assert!(clock.1.contains_rect(clock.0),"大号时刻不能被提醒条裁切");
        assert_eq!(countdown.1,egui::Color32::WHITE,"倒计时使用白色，强调色保留给来源 Logo");
        let title=h.shapes.iter().find_map(|s|match &s.shape{egui::Shape::Text(t) if t.galley.text()=="产品评审"=>Some((t.pos.x,t.galley.job.sections[0].format.font_id.size)),_=>None}).unwrap();
        assert!(clock.3>=28.0 && clock.3>title.1,"开始时刻应比事项名称更醒目");
        assert!((title.0-clock.2).abs()<0.5,"开始时刻与标题应沿同一列对齐");
        let date=h.shapes.iter().find_map(|s|match &s.shape{egui::Shape::Text(t) if t.galley.text().contains('月') && t.galley.text().contains('日')=>Some((t.visual_bounding_rect(),t.pos.x)),_=>None}).unwrap();
        let button=h.ctx.read_response(egui::Id::new(("card",true,"meeting-action"))).unwrap().rect;
        assert!((date.0.center().y-button.center().y).abs()<0.5,"日期与查看按钮应垂直居中对齐");
        assert!(((date.1-clock.1.left())-(clock.1.right()-button.right())).abs()<0.5,"卡片左右应保留相同边距");
    }
}

#[test]
fn meeting_long_titles_scroll_to_the_end_while_short_titles_stay_still(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    for title in ["简短评审","产品设计方案评审及下一阶段开发排期与交付验收安排确认"]{
        let mut h=H::new();h.app.settings.pet=false;
        h.app.meeting_alert(crate::mail::Meeting{ all_day:false,title:title.into(),start:chrono::Local::now().timestamp()+600,end:None,at:String::new(),uid:title.into()},"飞书".into(),7);
        for _ in 0..45{h.frame(vec![]);}
        let title_shape=|h:&H|h.shapes.iter().find_map(|s|match &s.shape{egui::Shape::Text(t) if t.galley.text()==title=>Some((t.pos.x,t.visual_bounding_rect(),s.clip_rect)),_=>None}).expect("标题应保留完整内容，不应提前省略");
        let initial=title_shape(&h);let height=h.hit.height();
        h.t+=0.2;h.frame(vec![]);assert!((title_shape(&h).0-initial.0).abs()<0.5,"开始滚动前应停留片刻");
        h.t+=5.0;h.frame(vec![]);let middle=title_shape(&h);
        h.t+=4.0;h.frame(vec![]);let end=title_shape(&h);
        if title=="简短评审" {assert!((end.0-initial.0).abs()<0.5,"短标题保持静止");}
        else {
            assert!(middle.0<initial.0-20.0,"长标题应向左滚动显示后半段");
            assert_eq!(initial.2,end.2,"滚动不应改变标题的显示范围");
            assert!(end.1.right()<=end.2.right()+1.0 && end.1.right()>=end.2.right()-5.0,"应能看到标题末尾");
            h.t+=0.6;h.frame(vec![]);assert!((title_shape(&h).0-end.0).abs()<0.5,"滚到末尾后应停留，不能继续滑出");
        }
        assert!((h.hit.height()-height).abs()<0.5,"滚动不应改变提醒高度");
    }
}

#[test]
fn calendar_notice_overrides_island_lyrics_and_restores_the_current_line(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    h.app.settings.pet=false;h.app.settings.island_lyrics=true;
    h.media=crate::smtc::NowPlaying{title:"Track".into(),artist:"Artist".into(),playing:true,position:7.0,duration:180.0,..Default::default()};
    h.app.set_lyrics("Track","Artist",180.0,crate::lyrics::parse_lrc("[00:00]首句歌词\n[00:20]后续歌词"));
    for _ in 0..60{h.frame(vec![]);}
    assert!(h.app.compact_notices());
    h.app.meeting_alert(crate::mail::Meeting{ all_day:false,title:"优先会议".into(),start:chrono::Local::now().timestamp()+600,end:None,at:String::new(),uid:"priority".into()},"飞书".into(),7);
    for _ in 0..90{h.frame(vec![]);}
    assert!(!h.app.compact_notices(),"日程提醒不应缩成歌词旁的小图标");
    assert!(h.hit.height()>130.0,"应直接展示完整 A 版提醒");
    assert!(h.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Text(t) if t.galley.text()=="优先会议")));
    assert!(h.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Text(t) if t.galley.text()=="查看")));
    assert!(!h.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Text(t) if t.galley.text()=="首句歌词")));
    let active=h.app.notice.as_ref().unwrap();h.t=active.at+active.duration-1.0/30.0;h.media.position=25.0;
    for _ in 0..90{
        h.frame(vec![]);
        if h.app.notice.is_none(){assert!(!h.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Text(t) if t.galley.text()=="Track" || t.galley.text()=="Artist")),"恢复歌词时不能闪出展开的播放器");}
    }
    assert!(h.app.notice.is_none() && h.app.settings.island_lyrics && h.app.compact_notices());
    assert!(h.hit.height()<35.0,"提醒结束后应恢复歌词条高度");
    assert!(h.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Text(t) if t.galley.text()=="后续歌词")),"应恢复当前进度的歌词");
}

#[test]
fn shelf_remove_icon_stays_centered_on_hover_and_preserves_the_file(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    for ppp in [1.0,1.25,1.5,2.0]{
        let mut h=H::new();h.ctx.set_pixels_per_point(ppp);
        let path=crate::model::config_dir().join("设计说明.txt");std::fs::create_dir_all(path.parent().unwrap()).unwrap();std::fs::write(&path,"保留原文件").unwrap();
        h.app.shelf.items=vec![path.clone()];h.app.show_page(3);
        for pointer in [H::local(300.0,60.0),H::local(233.0,39.0)]{
            h.hover(pointer,1.0);
            let button=h.ctx.read_response(egui::Id::new(("card",true,("shelf-remove",&path)))).unwrap().rect;
            let ink=h.shapes.iter().filter_map(|s|match &s.shape{
                egui::Shape::LineSegment{points,..} if points.iter().all(|p|button.shrink(2.0).contains(*p)) && (points[0].x-points[1].x).abs()>2.0 && (points[0].y-points[1].y).abs()>2.0=>Some(Rect::from_two_pos(points[0],points[1])),_=>None,
            }).fold(Rect::NOTHING,|a,b|a.union(b));
            assert!(ink.is_positive() && ink.center().distance(button.center())<0.4,"置物架移除图案应与按钮同心");
        }
        h.click(H::local(233.0,39.0));assert!(h.app.shelf.items.is_empty());assert!(path.exists());
    }
}

#[test]
fn resident_components_can_be_dragged_saved_and_reset(){
    use crate::bar_layout::{CLOCK,BATTERY};
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    fn draw(h:&mut H,events:Vec<egui::Event>)->[Pos2;2]{
        let mut points=[Pos2::ZERO;2];
        h.t+=0.02;
        let out=h.ctx.run(egui::RawInput{screen_rect:Some(Rect::from_min_size(Pos2::ZERO,egui::vec2(600.0,300.0))),time:Some(h.t),events,..Default::default()},|ctx|{egui::CentralPanel::default().show(ctx,|ui|{h.app.resident_order_settings(ui);for (index,slot) in [CLOCK,BATTERY].into_iter().enumerate(){points[index]=ui.ctx().read_response(ui.id().with(("resident-icon",slot))).unwrap().rect.center();}});});
        h.shapes=out.shapes;points
    }
    fn point(h:&H,label:&str)->Pos2{h.shapes.iter().find_map(|s|match &s.shape{egui::Shape::Text(t) if t.galley.text()==label=>Some(t.visual_bounding_rect().center()),_=>None}).unwrap()}
    draw(&mut h,vec![]);let [from,to]=draw(&mut h,vec![]);
    let button=|pos,pressed|egui::Event::PointerButton{pos,button:egui::PointerButton::Primary,pressed,modifiers:Default::default()};
    draw(&mut h,vec![egui::Event::PointerMoved(from),button(from,true)]);
    draw(&mut h,vec![egui::Event::PointerMoved(from+egui::vec2(0.0,10.0))]);
    draw(&mut h,vec![egui::Event::PointerMoved(to)]);
    for _ in 0..3{draw(&mut h,vec![]);}
    draw(&mut h,vec![button(to,false)]);
    assert_eq!(h.app.settings.resident_order.first(),Some(&CLOCK));
    assert!(h.app.settings.resident_order.contains(&BATTERY));
    h.app.settings.save();assert_eq!(crate::model::Settings::load().resident_order,h.app.settings.resident_order);
    let reset=point(&h,"恢复默认");draw(&mut h,vec![egui::Event::PointerMoved(reset),button(reset,true)]);draw(&mut h,vec![button(reset,false)]);
    assert!(h.app.settings.resident_order.is_empty());
}

#[test]
fn mailbox_scroll_reaches_older_mail_without_changing_pages() {
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    h.app.mail.accounts=vec![crate::mail::Account{email:"test@qq.com".into(),provider:crate::mail::Provider::Qq,enabled:true}];
    h.app.mail.recent=(1..=8).map(|uid|crate::mail::Item{account:"test@qq.com".into(),from:format!("来源 {uid}"),subject:format!("邮件 {uid}"),time:"11-03 09:41".into(),code:Some(format!("{uid:06}")),summary:String::new(),links:vec![],meeting:None,uid}).collect();
    h.app.show_page(5);h.hover(H::local(200.0,90.0),1.0);
    h.frame(vec![egui::Event::MouseWheel{unit:egui::MouseWheelUnit::Point,delta:egui::vec2(0.0,-900.0),modifiers:Default::default()}]);
    h.hover(H::local(200.0,90.0),0.6);
    assert_eq!(h.app.page(),5,"滚动邮箱列表不能切换页面");
    let row=h.ctx.read_response(egui::Id::new(("card",true,("mail-row",8u32,"test@qq.com")))).expect("第八封邮件应可访问");
    h.click(row.rect.center());
    assert_eq!(h.app.mail.open_uid,Some(("test@qq.com".into(),8)));
}

#[test]
#[ignore="按需检查邮箱列表和验证码来源的实际画面"]
fn mailbox_readability_gallery(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    let dest=std::path::PathBuf::from(std::env::var_os("DAOZAI_MAIL_PREVIEW").expect("需指定预览目录"));std::fs::create_dir_all(&dest).unwrap();
    for variant in 0..3 {
        let multiple=variant==1;let detail=variant==2;
        let mut h=H::new();h.app.settings.pet=false;
        h.app.mail.accounts=vec![crate::mail::Account{email:"work@qq.com".into(),provider:crate::mail::Provider::Qq,enabled:true}];
        if multiple {h.app.mail.accounts.push(crate::mail::Account{email:"personal@gmail.com".into(),provider:crate::mail::Provider::Gmail,enabled:true});}
        h.app.mail.recent=[("GitHub","登录验证码 · 本次登录有效期 10 分钟",Some("274630")),("Figma","产品设计评审 · 查看本轮修改意见",None),("Notion","确认你的邮箱地址",Some("815926")),("产品团队","周五项目安排与交付清单",None),("Microsoft","你的账户安全通知",None)].into_iter().enumerate().map(|(i,(from,subject,code))|crate::mail::Item{account:if multiple&&i%2==1{"personal@gmail.com"}else{"work@qq.com"}.into(),from:from.into(),subject:subject.into(),time:format!("11-03 09:{:02}",41-i),code:code.map(str::to_string),summary:String::new(),links:vec![],meeting:None,uid:i as u32}).collect();
        if detail {
            h.app.mail.recent[0].summary="你好，\n\n你的登录验证码是 274630，有效期为 10 分钟。\n\n如果这不是你本人的操作，请忽略这封邮件。请勿将验证码透露给他人。\n\n这封邮件由系统自动发送，无需回复。".into();
            h.app.mail.open_uid=Some(("work@qq.com".into(),0));
        }
        h.app.show_page(5);
        let ppp=1.5;let mut raster=crate::preview_raster::Raster::new((crate::app::CANVAS_W*ppp)as usize,(WIN_H*ppp)as usize);
        for frame in 0..100 {
            let t=frame as f64/60.0;let env=Env{t,clock:"09:41:00".into(),cursor:Some(H::local(200.0,90.0)),..Default::default()};
            let raw=egui::RawInput{screen_rect:Some(Rect::from_min_size(Pos2::ZERO,egui::vec2(crate::app::CANVAS_W,WIN_H))),time:Some(t),viewports:std::iter::once((egui::ViewportId::ROOT,egui::ViewportInfo{native_pixels_per_point:Some(ppp),..Default::default()})).collect(),..Default::default()};
            let out=h.ctx.run(raw,|ctx|{egui::CentralPanel::default().frame(egui::Frame::new()).show(ctx,|ui|{h.app.frame(ui,Rect::from_min_size(Pos2::ZERO,egui::vec2(WIN_W,WIN_H)),&env);});});
            raster.update_textures(&out.textures_delta);
            if frame==99 {raster.buf.fill([0.05,0.07,0.10,1.0]);raster.draw(&h.ctx.tessellate(out.shapes,out.pixels_per_point),out.pixels_per_point);let mut rgba=Vec::new();raster.to_rgba8(&mut rgba);let image=image::RgbaImage::from_raw((crate::app::CANVAS_W*ppp)as u32,(WIN_H*ppp)as u32,rgba).unwrap();image::imageops::crop_imm(&image,((WIN_W-PW)/2.0*ppp-10.0)as u32,0,(PW*ppp+20.0)as u32,if detail{370}else{225}).to_image().save(dest.join(if detail{"邮件详情.png"}else if multiple{"多账号邮箱.png"}else{"邮箱列表.png"})).unwrap();}
            raster.free_textures(&out.textures_delta);
        }
    }
}

#[test]
fn mailbox_large_code_card_preserves_copy_and_identifies_source(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    h.app.mail.accounts=["work@qq.com","personal@gmail.com"].into_iter().map(|email|crate::mail::Account{email:email.into(),provider:crate::mail::Provider::Qq,enabled:true}).collect();
    h.app.mail.recent=[("work@qq.com","GitHub","274630"),("personal@gmail.com","Notion","815926")].into_iter().map(|(account,from,code)|crate::mail::Item{account:account.into(),from:from.into(),subject:"登录验证码".into(),time:"09:41".into(),code:Some(code.into()),summary:String::new(),links:vec![],meeting:None,uid:42}).collect();
    h.app.show_page(5);h.hover(H::local(200.0,90.0),1.0);
    assert!(h.shapes.iter().any(|shape|matches!(&shape.shape,egui::Shape::Text(t) if t.galley.text()=="GitHub")));
    h.cmds.clear();h.click(H::local(60.0,70.0));
    assert!(h.cmds.iter().any(|cmd|matches!(cmd,crate::app::Cmd::Copy(value) if value=="274630")));
    assert!(h.app.mail.open_uid.is_none(),"复制验证码不能误开邮件详情");
    assert_eq!(h.app.expanded_height(),PH,"邮箱沿用录屏里的面板尺寸");
}

#[test]
fn hidden_components_do_not_keep_static_pages_animating(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    h.app.settings.pet=false;
    h.app.show_page(4);h.hover(H::local(300.0,60.0),2.0);
    assert!(!h.animating);
    let mut moving=crate::anim::Spring::new(0.0,230.0,0.9);moving.target=200.0;
    h.app.shelf_x.insert("hidden.txt".into(),moving);
    h.frame(vec![]);
    assert!(!h.animating,"离开置物页后，未完成的文件动画不能让其他页面持续刷新");
    h.app.shelf_x.clear();h.app.pomo.running=true;
    h.hover(H::local(300.0,60.0),1.5);
    assert!(!h.animating,"待办页没有显示专注动画，计时继续但不应强制连续刷新");
}

#[test]
fn minimal_music_stops_hidden_playback_animations(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    h.app.set_minimal(true);
    h.media=crate::smtc::NowPlaying{title:"歌曲".into(),artist:"歌手".into(),playing:true,..Default::default()};
    for _ in 0..180{h.frame(vec![]);}
    assert!(!h.app.open);
    assert!(!h.animating,"极简模式隐藏媒体组件后，不应继续为播放动画刷新");
}

#[test]
fn native_app_icon_uses_bounded_resolution(){
    let icon=crate::theme::app_icon();
    assert!(icon.width<=128 && icon.height<=128,"原生窗口图标不需要常驻完整的 1024 像素品牌原图");
    assert_eq!(icon.rgba.len(),(icon.width*icon.height*4)as usize);
    assert!(std::sync::Arc::ptr_eq(&icon,&crate::theme::app_icon()));
}

#[test]
fn long_mail_lists_only_layout_the_visible_rows(){
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    h.app.settings.pet=false;
    h.app.mail.accounts=vec![crate::mail::Account{email:"work@qq.com".into(),provider:crate::mail::Provider::Qq,enabled:true}];
    h.app.mail.recent=(0..60).map(|i|crate::mail::Item{account:"work@qq.com".into(),from:format!("发件人 {i}"),subject:format!("第 {i} 封邮件标题"),time:"09:41".into(),code:None,summary:String::new(),links:vec![],meeting:None,uid:i}).collect();
    h.app.show_page(5);h.hover(H::local(300.0,60.0),1.0);
    let count=h.ctx.fonts(|fonts|fonts.num_galleys_in_cache());
    assert!(count<100,"60 封邮件不应全部排版，当前缓存了 {count} 个文字布局");
    for _ in 0..15{h.wheel(-40.0);h.hover(H::local(300.0,60.0),0.1);}
    assert_eq!(h.app.page,5,"列表虚拟化后仍应在邮箱页滚动");
}

#[test]
fn calendar_overnight_appears_on_both_days_but_not_after_end() {
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    let first=chrono::NaiveDate::from_ymd_opt(2026,11,3).unwrap();
    h.app.calendar.view.loaded=Some(crate::calendar::month_start(first));
    let start=first.and_hms_opt(23,30,0).unwrap().and_local_timezone(chrono::Local).earliest().unwrap().timestamp();
    h.app.calendar.view.events.push(crate::mail::Meeting{ all_day:false,title:"跨夜".into(),start,end:Some(start+3600),at:String::new(),uid:"overnight".into()});
    assert_eq!(h.app.calendar_entries(first).len(),1);
    assert_eq!(h.app.calendar_entries(first.succ_opt().unwrap()).len(),1);
    assert_eq!(h.app.calendar_entries(first+chrono::Duration::days(2)).len(),0);
}

#[test]
fn calendar_month_refresh_removes_cancelled_cached_event() {
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    let day=chrono::NaiveDate::from_ymd_opt(2026,11,3).unwrap();
    let start=day.and_hms_opt(16,0,0).unwrap().and_local_timezone(chrono::Local).earliest().unwrap().timestamp();
    h.app.calendar.events.push(crate::mail::Meeting{all_day:false,title:"已取消".into(),start,end:None,at:String::new(),uid:"removed".into()});
    h.app.calendar.view.loaded=Some(crate::calendar::month_start(day));
    h.app.calendar.view.events.clear();
    assert!(h.app.calendar_entries(day).is_empty(),"月份同步后不能让提醒缓存恢复已取消日程");
}

#[test]
fn ignored_mail_event_hides_stops_reminders_persists_and_restores() {
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    let now=chrono::Local::now();let day=now.date_naive();
    let meeting=crate::mail::Meeting{all_day:false,title:"测试邀请".into(),start:now.timestamp()+600,end:None,at:String::new(),uid:"ignore-me".into()};
    h.app.mail.recent.push(crate::mail::Item{account:"test@qq.com".into(),from:"测试".into(),subject:"邀请".into(),time:String::new(),code:None,summary:String::new(),links:vec![],meeting:Some(meeting.clone()),uid:99});
    h.app.toggle_mail_event(&meeting);
    assert!(h.app.calendar_entries(day).is_empty());
    assert!(crate::model::Settings::load().ignored_mail_events.contains(&crate::calendar::dedupe_key(&meeting)));
    h.frame(vec![]);assert!(!h.app.history.iter().any(|a|a.title=="会议提醒"));
    assert_eq!(h.app.mail.recent.len(),1,"忽略不删除邮件");
    h.app.calendar.events.push(meeting.clone());
    assert_eq!(h.app.calendar_entries(day)[0].sources.len(),1,"飞书来源不受邮件忽略影响");
    h.app.calendar.events.clear();h.app.toggle_mail_event(&meeting);
    assert_eq!(h.app.calendar_entries(day).len(),1);
    h.frame(vec![]);assert!(h.app.history.iter().any(|a|a.title=="会议提醒"));
}

#[test]
fn calendar_sync_result_does_not_enqueue_another_sync() {
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    let config=crate::calendar::Config{provider:crate::calendar::Provider::Feishu,username:"test".into(),href:None,server:None};
    h.app.calendar.config=Some(config.clone());h.app.open_calendar(None);h.app.pending.clear();
    h.app.calendar.view.loading=false;
    h.app.calendar_event(crate::calendar::Event::Synced{config,events:vec![],at:"12:00".into()});
    assert!(!h.app.pending.iter().any(|cmd|matches!(cmd,crate::app::Cmd::CalendarMonth(_))),"同步结果不能触发下一轮同步，否则后台会持续请求");
}

#[test]
fn page_order_persists_and_controls_navigation() {
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());
    let mut h=H::new();
    h.app.settings.page_order=[5,2,0,1,3,4,6,7];
    h.app.settings.save();
    let settings=crate::model::Settings::load();
    assert_eq!(settings.page_order,[5,2,0,1,3,4,6,7]);
    let restarted=App::new(&h.ctx,Options{fetch_lyrics:false,pet_idle:1e9});
    assert_eq!(restarted.page(),5);
    for page in [1,3,4,6,7] {h.app.set_page_visible(page,false);}
    h.hover(Pos2::new(WIN_W/2.0,12.0),1.0);
    for expected in [5,2,0] {
        h.click(H::local(398.0,68.0));
        assert_eq!(h.app.page,expected);
        h.hover(H::local(300.0,60.0),0.7);
    }
    h.click(H::local(21.0,68.0));
    assert_eq!(h.app.page,2);
    h.app.set_page_visible(2,false);
    assert_eq!(h.app.page,5);
    for invalid in ["0,0,1,2,3,4,5,6","8,1,2,3,4,5,6,7","0,1","invalid"] {
        std::fs::write(crate::model::config_dir().join("settings.txt"),format!("page_order={invalid}\n")).unwrap();
        assert_eq!(crate::model::Settings::load().page_order,[0,1,2,3,4,5,6,7]);
    }
}

#[test]
fn page_order_drag_applies_on_release_and_saves() {
    let _g=ENV_LOCK.lock().unwrap_or_else(|e|e.into_inner());let mut h=H::new();
    h.app.settings_tab=10;h.app.open_settings();h.ctx.set_embed_viewports(false);
    fn draw(h:&mut H,events:Vec<egui::Event>){
        let output=std::sync::Arc::new(Mutex::new(None));let capture=output.clone();
        h.t+=0.02;let time=h.t;
        egui::Context::set_immediate_viewport_renderer(move|ctx,mut viewport|{
            let id=viewport.ids.this;
            let raw=egui::RawInput{viewport_id:id,screen_rect:Some(Rect::from_min_size(Pos2::ZERO,egui::vec2(690.0,1000.0))),time:Some(time),events:events.clone(),viewports:std::iter::once((id,egui::ViewportInfo::default())).collect(),..Default::default()};
            *capture.lock().unwrap()=Some(ctx.run(raw,|ctx|(viewport.viewport_ui_cb)(ctx)));
        });
        let _=h.ctx.run(egui::RawInput::default(),|ctx|h.app.settings_ui(ctx,false));
        let child=output.lock().unwrap().take().unwrap();h.shapes=child.shapes;
    }
    for _ in 0..3 {draw(&mut h,vec![]);}
    let point=|h:&H,label:&str|h.shapes.iter().find_map(|s|match &s.shape {egui::Shape::Text(t) if t.galley.text()==label=>Some(t.visual_bounding_rect().left_center()),_=>None}).unwrap();
    let from=point(&h,"播放控制与同步歌词")+egui::vec2(-18.0,-6.0);
    let to=point(&h,"饮水记录与提醒");
    let button=|pos,pressed|egui::Event::PointerButton{pos,button:egui::PointerButton::Primary,pressed,modifiers:Default::default()};
    draw(&mut h,vec![egui::Event::PointerMoved(from),button(from,true)]);
    draw(&mut h,vec![egui::Event::PointerMoved(from+egui::vec2(0.0,10.0))]);
    draw(&mut h,vec![egui::Event::PointerMoved(to)]);
    assert_eq!(h.app.settings.page_order,[0,1,2,3,4,5,6,7]);
    draw(&mut h,vec![button(to,false)]);
    assert_eq!(h.app.settings.page_order,[1,2,0,3,4,5,6,7]);
    assert_eq!(crate::model::Settings::load().page_order,h.app.settings.page_order);
}
