use std::sync::Arc;

use elements_ecs::{EntityId, World};
use elements_input::MouseButton;
use elements_scripting_host::shared::host_guest_state::BaseHostGuestState;
use elements_ui::VirtualKeyCode;
use glam::Vec2;
use parking_lot::RwLock;
use tilt_runtime_core::player::RawInput;

use super::{implementation as trsi, interface::host};

pub struct Bindings {
    shared_state: Arc<RwLock<BaseHostGuestState>>,
}
impl Bindings {
    pub const fn new(shared_state: Arc<RwLock<BaseHostGuestState>>) -> Self {
        Self { shared_state }
    }
    fn world(&self) -> parking_lot::MappedRwLockReadGuard<World> {
        parking_lot::RwLockReadGuard::map(self.shared_state.read(), |s| s.world())
    }
}

impl host::Host for Bindings {
    fn player_get_raw_input(&mut self, player: host::EntityId) -> Option<host::PlayerRawInput> {
        trsi::player::get_raw_input(&self.world(), player.from_tilt_runtime_bindgen())
            .into_tilt_runtime_bindgen()
    }

    fn player_get_prev_raw_input(
        &mut self,
        player: host::EntityId,
    ) -> Option<host::PlayerRawInput> {
        trsi::player::get_prev_raw_input(&self.world(), player.from_tilt_runtime_bindgen())
            .into_tilt_runtime_bindgen()
    }
}

/// Converts from a Rust representation to a wit-bindgen representation.
trait IntoTiltRuntimeBindgen {
    type Item;
    fn into_tilt_runtime_bindgen(self) -> Self::Item;
}

impl<T> IntoTiltRuntimeBindgen for Option<T>
where
    T: IntoTiltRuntimeBindgen,
{
    type Item = Option<T::Item>;
    fn into_tilt_runtime_bindgen(self) -> Self::Item {
        self.map(|i| i.into_tilt_runtime_bindgen())
    }
}

/// Converts from a wit-bindgen representation to a Rust representation.
#[allow(clippy::wrong_self_convention)]
trait FromTiltRuntimeBindgen {
    type Item;
    fn from_tilt_runtime_bindgen(self) -> Self::Item;
}

impl FromTiltRuntimeBindgen for host::EntityId {
    type Item = EntityId;
    fn from_tilt_runtime_bindgen(self) -> Self::Item {
        EntityId {
            namespace: self.namespace,
            id: self.id as usize,
            gen: self.gen,
        }
    }
}

impl IntoTiltRuntimeBindgen for Vec2 {
    type Item = host::Vec2;
    fn into_tilt_runtime_bindgen(self) -> Self::Item {
        host::Vec2 {
            x: self.x,
            y: self.y,
        }
    }
}

impl IntoTiltRuntimeBindgen for RawInput {
    type Item = host::PlayerRawInput;

    fn into_tilt_runtime_bindgen(self) -> Self::Item {
        Self::Item {
            keys: self
                .keys
                .into_iter()
                .map(|k| k.into_tilt_runtime_bindgen())
                .collect(),
            mouse_position: self.mouse_position.into_tilt_runtime_bindgen(),
            mouse_wheel: self.mouse_wheel,
            mouse_buttons: self
                .mouse_buttons
                .into_iter()
                .map(|b| b.into_tilt_runtime_bindgen())
                .collect(),
        }
    }
}

impl IntoTiltRuntimeBindgen for VirtualKeyCode {
    type Item = host::VirtualKeyCode;

    fn into_tilt_runtime_bindgen(self) -> Self::Item {
        match self {
            Self::Key1 => Self::Item::Key1,
            Self::Key2 => Self::Item::Key2,
            Self::Key3 => Self::Item::Key3,
            Self::Key4 => Self::Item::Key4,
            Self::Key5 => Self::Item::Key5,
            Self::Key6 => Self::Item::Key6,
            Self::Key7 => Self::Item::Key7,
            Self::Key8 => Self::Item::Key8,
            Self::Key9 => Self::Item::Key9,
            Self::Key0 => Self::Item::Key0,
            Self::A => Self::Item::A,
            Self::B => Self::Item::B,
            Self::C => Self::Item::C,
            Self::D => Self::Item::D,
            Self::E => Self::Item::E,
            Self::F => Self::Item::F,
            Self::G => Self::Item::G,
            Self::H => Self::Item::H,
            Self::I => Self::Item::I,
            Self::J => Self::Item::J,
            Self::K => Self::Item::K,
            Self::L => Self::Item::L,
            Self::M => Self::Item::M,
            Self::N => Self::Item::N,
            Self::O => Self::Item::O,
            Self::P => Self::Item::P,
            Self::Q => Self::Item::Q,
            Self::R => Self::Item::R,
            Self::S => Self::Item::S,
            Self::T => Self::Item::T,
            Self::U => Self::Item::U,
            Self::V => Self::Item::V,
            Self::W => Self::Item::W,
            Self::X => Self::Item::X,
            Self::Y => Self::Item::Y,
            Self::Z => Self::Item::Z,
            Self::Escape => Self::Item::Escape,
            Self::F1 => Self::Item::F1,
            Self::F2 => Self::Item::F2,
            Self::F3 => Self::Item::F3,
            Self::F4 => Self::Item::F4,
            Self::F5 => Self::Item::F5,
            Self::F6 => Self::Item::F6,
            Self::F7 => Self::Item::F7,
            Self::F8 => Self::Item::F8,
            Self::F9 => Self::Item::F9,
            Self::F10 => Self::Item::F10,
            Self::F11 => Self::Item::F11,
            Self::F12 => Self::Item::F12,
            Self::F13 => Self::Item::F13,
            Self::F14 => Self::Item::F14,
            Self::F15 => Self::Item::F15,
            Self::F16 => Self::Item::F16,
            Self::F17 => Self::Item::F17,
            Self::F18 => Self::Item::F18,
            Self::F19 => Self::Item::F19,
            Self::F20 => Self::Item::F20,
            Self::F21 => Self::Item::F21,
            Self::F22 => Self::Item::F22,
            Self::F23 => Self::Item::F23,
            Self::F24 => Self::Item::F24,
            Self::Snapshot => Self::Item::Snapshot,
            Self::Scroll => Self::Item::Scroll,
            Self::Pause => Self::Item::Pause,
            Self::Insert => Self::Item::Insert,
            Self::Home => Self::Item::Home,
            Self::Delete => Self::Item::Delete,
            Self::End => Self::Item::End,
            Self::PageDown => Self::Item::PageDown,
            Self::PageUp => Self::Item::PageUp,
            Self::Left => Self::Item::Left,
            Self::Up => Self::Item::Up,
            Self::Right => Self::Item::Right,
            Self::Down => Self::Item::Down,
            Self::Back => Self::Item::Back,
            Self::Return => Self::Item::Return,
            Self::Space => Self::Item::Space,
            Self::Compose => Self::Item::Compose,
            Self::Caret => Self::Item::Caret,
            Self::Numlock => Self::Item::Numlock,
            Self::Numpad0 => Self::Item::Numpad0,
            Self::Numpad1 => Self::Item::Numpad1,
            Self::Numpad2 => Self::Item::Numpad2,
            Self::Numpad3 => Self::Item::Numpad3,
            Self::Numpad4 => Self::Item::Numpad4,
            Self::Numpad5 => Self::Item::Numpad5,
            Self::Numpad6 => Self::Item::Numpad6,
            Self::Numpad7 => Self::Item::Numpad7,
            Self::Numpad8 => Self::Item::Numpad8,
            Self::Numpad9 => Self::Item::Numpad9,
            Self::NumpadAdd => Self::Item::NumpadAdd,
            Self::NumpadDivide => Self::Item::NumpadDivide,
            Self::NumpadDecimal => Self::Item::NumpadDecimal,
            Self::NumpadComma => Self::Item::NumpadComma,
            Self::NumpadEnter => Self::Item::NumpadEnter,
            Self::NumpadEquals => Self::Item::NumpadEquals,
            Self::NumpadMultiply => Self::Item::NumpadMultiply,
            Self::NumpadSubtract => Self::Item::NumpadSubtract,
            Self::AbntC1 => Self::Item::AbntC1,
            Self::AbntC2 => Self::Item::AbntC2,
            Self::Apostrophe => Self::Item::Apostrophe,
            Self::Apps => Self::Item::Apps,
            Self::Asterisk => Self::Item::Asterisk,
            Self::At => Self::Item::At,
            Self::Ax => Self::Item::Ax,
            Self::Backslash => Self::Item::Backslash,
            Self::Calculator => Self::Item::Calculator,
            Self::Capital => Self::Item::Capital,
            Self::Colon => Self::Item::Colon,
            Self::Comma => Self::Item::Comma,
            Self::Convert => Self::Item::Convert,
            Self::Equals => Self::Item::Equals,
            Self::Grave => Self::Item::Grave,
            Self::Kana => Self::Item::Kana,
            Self::Kanji => Self::Item::Kanji,
            Self::LAlt => Self::Item::LAlt,
            Self::LBracket => Self::Item::LBracket,
            Self::LControl => Self::Item::LControl,
            Self::LShift => Self::Item::LShift,
            Self::LWin => Self::Item::LWin,
            Self::Mail => Self::Item::Mail,
            Self::MediaSelect => Self::Item::MediaSelect,
            Self::MediaStop => Self::Item::MediaStop,
            Self::Minus => Self::Item::Minus,
            Self::Mute => Self::Item::Mute,
            Self::MyComputer => Self::Item::MyComputer,
            Self::NavigateForward => Self::Item::NavigateForward,
            Self::NavigateBackward => Self::Item::NavigateBackward,
            Self::NextTrack => Self::Item::NextTrack,
            Self::NoConvert => Self::Item::NoConvert,
            Self::OEM102 => Self::Item::Oem102,
            Self::Period => Self::Item::Period,
            Self::PlayPause => Self::Item::PlayPause,
            Self::Plus => Self::Item::Plus,
            Self::Power => Self::Item::Power,
            Self::PrevTrack => Self::Item::PrevTrack,
            Self::RAlt => Self::Item::RAlt,
            Self::RBracket => Self::Item::RBracket,
            Self::RControl => Self::Item::RControl,
            Self::RShift => Self::Item::RShift,
            Self::RWin => Self::Item::RWin,
            Self::Semicolon => Self::Item::Semicolon,
            Self::Slash => Self::Item::Slash,
            Self::Sleep => Self::Item::Sleep,
            Self::Stop => Self::Item::Stop,
            Self::Sysrq => Self::Item::Sysrq,
            Self::Tab => Self::Item::Tab,
            Self::Underline => Self::Item::Underline,
            Self::Unlabeled => Self::Item::Unlabeled,
            Self::VolumeDown => Self::Item::VolumeDown,
            Self::VolumeUp => Self::Item::VolumeUp,
            Self::Wake => Self::Item::Wake,
            Self::WebBack => Self::Item::WebBack,
            Self::WebFavorites => Self::Item::WebFavorites,
            Self::WebForward => Self::Item::WebForward,
            Self::WebHome => Self::Item::WebHome,
            Self::WebRefresh => Self::Item::WebRefresh,
            Self::WebSearch => Self::Item::WebSearch,
            Self::WebStop => Self::Item::WebStop,
            Self::Yen => Self::Item::Yen,
            Self::Copy => Self::Item::Copy,
            Self::Paste => Self::Item::Paste,
            Self::Cut => Self::Item::Cut,
        }
    }
}

impl IntoTiltRuntimeBindgen for MouseButton {
    type Item = host::MouseButton;

    fn into_tilt_runtime_bindgen(self) -> Self::Item {
        match self {
            Self::Left => Self::Item::Left,
            Self::Right => Self::Item::Right,
            Self::Middle => Self::Item::Middle,
            Self::Other(id) => Self::Item::Other(id),
        }
    }
}
