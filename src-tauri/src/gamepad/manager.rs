use gilrs_core::{Event, EventType, Gilrs};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime};
use tauri::{AppHandle, Emitter};

const AXIS_MIN_VALUE: i32 = -32768;
const AXIS_MAX_VALUE: i32 = 32767;

// TODO:リリースタイムは直近300ノーツを対象に計算するようにする
struct ButtonEvent {
    pressed_count: u32,
    pressed_time: Option<SystemTime>,
}

impl ButtonEvent {
    fn new() -> ButtonEvent {
        ButtonEvent {
            pressed_count: 0,
            pressed_time: None,
        }
    }

    fn on_press(&mut self) {
        self.pressed_count += 1;
        self.pressed_time = Some(SystemTime::now());
    }

    fn on_release(&mut self) -> Option<Duration> {
        if let Some(press_time) = self.pressed_time.take() {
            if let Ok(duration) = press_time.elapsed() {
                if duration < Duration::from_millis(200) {
                    return Some(duration);
                } else {
                    return None;
                }
            }
        }
        None
    }
}

#[derive(PartialEq, Clone)]
enum Direction {
    Neutral,
    Left,
    Right,
}
struct ScratchEvent {
    spined_time: Option<Instant>,
    direction: Direction,
    axis: i32,
}

impl ScratchEvent {
    // TODO:初期値をGilrsのGamepadから取得する
    fn new() -> ScratchEvent {
        ScratchEvent {
            spined_time: None,
            direction: Direction::Neutral,
            axis: 0,
        }
    }

    fn on_spin(&mut self, new_axis: i32) -> Option<Direction> {
        let mut new_direction = Direction::Neutral;
        if new_axis == AXIS_MIN_VALUE && self.axis == AXIS_MAX_VALUE {
            new_direction = Direction::Left;
        } else if new_axis == AXIS_MAX_VALUE && self.axis == AXIS_MIN_VALUE {
            new_direction = Direction::Right;
        } else if new_axis > self.axis {
            new_direction = Direction::Left;
        } else if new_axis < self.axis {
            new_direction = Direction::Right;
        }

        println!("axis:{}   new_axis:{}", self.axis, new_axis);
        self.axis = new_axis;
        self.spined_time = Some(Instant::now());

        if self.direction == new_direction {
            return None;
        } else {
            self.direction = new_direction.clone();
            return Some(new_direction);
        }
    }

    fn reset_to_neutral(&mut self) -> bool {
        if self.direction != Direction::Neutral {
            self.direction = Direction::Neutral;
            self.spined_time = None;
            return true;
        }
        false
    }
    fn is_timeout(&self) -> bool {
        if let Some(spined_time) = self.spined_time {
            if self.direction != Direction::Neutral {
                return spined_time.elapsed() >= Duration::from_millis(100);
            }
        }
        false
    }
}

pub struct GamepadStatus {
    button_status: HashMap<u32, ButtonEvent>,
    recently_release_time: VecDeque<Duration>,
    scratch_event: ScratchEvent,
}

impl GamepadStatus {
    fn new() -> GamepadStatus {
        GamepadStatus {
            button_status: HashMap::new(),
            recently_release_time: VecDeque::new(),
            scratch_event: ScratchEvent::new(),
        }
    }

    fn on_press(&mut self, button_code: u32) {
        let button_event = self
            .button_status
            .entry(button_code)
            .or_insert_with(ButtonEvent::new);
        button_event.on_press();
    }
    fn on_release(&mut self, button_code: u32) -> Duration {
        if let Some(button_event) = self.button_status.get_mut(&button_code) {
            let duration = button_event.on_release();
            if let Some(release_time) = duration {
                self.recently_release_time.push_back(release_time);
                if self.recently_release_time.len() > 2000 {
                    self.recently_release_time.pop_front();
                }
            }
        };
        return self.calc_release_time();
    }
    // TODO: recently_release_timeの配列の配列が空であるときのエラー処理を書く
    fn calc_release_time(&self) -> Duration {
        let mut release_time = Duration::from_millis(0);
        let num = self.recently_release_time.len();
        let sum = self
            .recently_release_time
            .iter()
            .fold(Duration::from_millis(0), |sum, x| sum + *x);
        if num > 0 {
            release_time = sum / num as u32;
        }

        return release_time;
    }
}

pub struct GamepadManager {
    pub gilrs: Arc<Mutex<Gilrs>>,
    pub active_gamepad: Arc<Mutex<Option<usize>>>,
    pub status: Arc<Mutex<GamepadStatus>>,
}

impl GamepadManager {
    pub fn new() -> GamepadManager {
        let gilrs = Gilrs::new().unwrap();

        let active_gamepad = None;
        GamepadManager {
            gilrs: Arc::new(Mutex::new(gilrs)),
            active_gamepad: Arc::new(Mutex::new(active_gamepad)),
            status: Arc::new(Mutex::new(GamepadStatus::new())),
        }
    }

    pub fn start_event_loop(&self, app_handle: AppHandle) {
        let gilrs = Arc::clone(&self.gilrs);
        let active_gamepad = Arc::clone(&self.active_gamepad);
        let status = Arc::clone(&self.status);

        let main_handle = app_handle.clone();
        let sub_handle = app_handle.clone();

        thread::spawn(move || {
            loop {
                // Examine new events
                if let Ok(mut gilrs) = gilrs.lock() {
                    while let Some(Event {
                        id, event, time, ..
                    }) = gilrs.next_event()
                    {
                        if let Ok(mut active_gamepad) = active_gamepad.lock() {
                            *active_gamepad = Some(id);
                        }
                        match event {
                            EventType::ButtonPressed(button) => {
                                let button_code = button.into_u32();

                                // statusの更新
                                if let Ok(mut status) = status.lock() {
                                    status.on_press(button_code);
                                }

                                let event = serde_json::json!({
                                    "type": "button",
                                    "button": button_code,
                                    "pressed": true,
                                });
                                main_handle.emit("gamepad-input", &event).unwrap();
                            }
                            EventType::ButtonReleased(button) => {
                                let button_code = button.into_u32();
                                let mut average_release_time = 0;

                                // statusの更新
                                if let Ok(mut status) = status.lock() {
                                    average_release_time =
                                        status.on_release(button_code).as_millis();
                                }

                                let event = serde_json::json!({
                                    "type": "button",
                                    "button": button_code,
                                    "pressed": false,
                                    "averageReleaseTime": average_release_time,
                                });
                                main_handle.emit("gamepad-input", &event).unwrap();
                            }
                            EventType::AxisValueChanged(axis, _) => {
                                if let Ok(mut status) = status.lock() {
                                    if let Some(direction) = status.scratch_event.on_spin(axis) {
                                        let event = serde_json::json!({
                                            "type": "scratch",
                                            "axis" : axis,
                                            "direction": match direction{
                                                Direction::Left => "left",
                                                Direction::Right => "right",
                                                Direction::Neutral => "neutral",
                                            }
                                        });
                                        main_handle.emit("gamepad-input", &event).unwrap();
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                thread::sleep(std::time::Duration::from_millis(8))
            }
        });

        // スクラッチタイムアウト用のスレッド
        let status = Arc::clone(&self.status);
        thread::spawn(move || loop {
            thread::sleep(Duration::from_millis(10));
            if let Ok(mut status) = status.lock() {
                if status.scratch_event.is_timeout() {
                    if status.scratch_event.reset_to_neutral() {
                        let event = serde_json::json!({
                            "type" :"scratch",
                            "axis" : status.scratch_event.axis,
                            "direction" : "neutral"
                        });

                        sub_handle.emit("gamepad-input", &event).unwrap();
                    }
                }
            }
        });
    }
}
