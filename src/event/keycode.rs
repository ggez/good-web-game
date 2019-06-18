use rustc_hash::FxHashMap;
use lazy_static::lazy_static;

lazy_static! {
    static ref CODES: FxHashMap<&'static str, KeyCode> = {
        use KeyCode::*;

        let mut codes = FxHashMap::default();

        codes.insert("Digit1", Key1);
        codes.insert("Digit2", Key2);
        codes.insert("Digit3", Key3);
        codes.insert("Digit4", Key4);
        codes.insert("Digit5", Key5);
        codes.insert("Digit6", Key6);
        codes.insert("Digit7", Key7);
        codes.insert("Digit8", Key8);
        codes.insert("Digit9", Key9);
        codes.insert("Digit0", Key0);
        codes.insert("KeyA", A);
        codes.insert("KeyB", B);
        codes.insert("KeyC", C);
        codes.insert("KeyD", D);
        codes.insert("KeyE", E);
        codes.insert("KeyF", F);
        codes.insert("KeyG", G);
        codes.insert("KeyH", H);
        codes.insert("KeyI", I);
        codes.insert("KeyJ", J);
        codes.insert("KeyK", K);
        codes.insert("KeyL", L);
        codes.insert("KeyM", M);
        codes.insert("KeyN", N);
        codes.insert("KeyO", O);
        codes.insert("KeyP", P);
        codes.insert("KeyQ", Q);
        codes.insert("KeyR", R);
        codes.insert("KeyS", S);
        codes.insert("KeyT", T);
        codes.insert("KeyU", U);
        codes.insert("KeyV", V);
        codes.insert("KeyW", W);
        codes.insert("KeyX", X);
        codes.insert("KeyY", Y);
        codes.insert("KeyZ", Z);
        codes.insert("Escape", Escape);
        codes.insert("F1", F1);
        codes.insert("F2", F2);
        codes.insert("F3", F3);
        codes.insert("F4", F4);
        codes.insert("F5", F5);
        codes.insert("F6", F6);
        codes.insert("F7", F7);
        codes.insert("F8", F8);
        codes.insert("F9", F9);
        codes.insert("F10", F10);
        codes.insert("F11", F11);
        codes.insert("F12", F12);
        codes.insert("F13", F13);
        codes.insert("F14", F14);
        codes.insert("F15", F15);
        codes.insert("F16", F16);
        codes.insert("F17", F17);
        codes.insert("F18", F18);
        codes.insert("F19", F19);
        codes.insert("F20", F20);
        codes.insert("F21", F21);
        codes.insert("F22", F22);
        codes.insert("F23", F23);
        codes.insert("F24", F24);
        codes.insert("PrintScreen", Snapshot);
        codes.insert("ScrollLock", Scroll);
        codes.insert("Pause", Pause);
        codes.insert("Insert", Insert);
        codes.insert("Home", Home);
        codes.insert("Delete", Delete);
        codes.insert("End", End);
        codes.insert("PageDown", PageDown);
        codes.insert("PageUp", PageUp);
        codes.insert("ArrowLeft", Left);
        codes.insert("ArrowUp", Up);
        codes.insert("ArrowRight", Right);
        codes.insert("ArrowDown", Down);
        codes.insert("Backspace", Back);
        codes.insert("Enter", Return);
        codes.insert("Space", Space);
        // Compose,
        // Caret,
        codes.insert("NumLock", Numlock);
        codes.insert("Numpad0", Numpad0);
        codes.insert("Numpad1", Numpad1);
        codes.insert("Numpad2", Numpad2);
        codes.insert("Numpad3", Numpad3);
        codes.insert("Numpad4", Numpad4);
        codes.insert("Numpad5", Numpad5);
        codes.insert("Numpad6", Numpad6);
        codes.insert("Numpad7", Numpad7);
        codes.insert("Numpad8", Numpad8);
        codes.insert("Numpad9", Numpad9);
        // AbntC1,
        // AbntC2,
        codes.insert("NumpadAdd", Add);
        codes.insert("Quote", Apostrophe);
        // Apps,
        // At,
        // Ax,
        codes.insert("Backslash", Backslash);
        // Calculator,
        // Capital,
        // Colon,
        codes.insert("Comma", Comma);
        codes.insert("Convert", Convert);
        codes.insert("NumpadDecimal", Decimal);
        codes.insert("NumpadDivide", Divide);
        codes.insert("Equal", Equals);
        codes.insert("Backquote", Grave);
        codes.insert("KanaMode", Kana);
        // Kanji,
        codes.insert("AltLeft", LAlt);
        codes.insert("BracketLeft", LBracket);
        codes.insert("ControlLeft", LControl);
        codes.insert("ShiftLeft", LShift);
        codes.insert("MetaLeft", LWin);
        codes.insert("OSLeft", LWin);
        codes.insert("LaunchMail", Mail);
        codes.insert("LaunchMediaPlayer", MediaSelect);
        codes.insert("MediaSelect", MediaSelect);
        codes.insert("MediaStop", MediaStop);
        codes.insert("Minus", Minus);
        codes.insert("NumpadMultiply", Multiply);
        codes.insert("AudioVolumeMute", Mute);
        // MyComputer,
        // NavigateForward,
        // NavigateBackward,
        codes.insert("MediaTrackNext", NextTrack);
        codes.insert("NonConvert", NoConvert);
        codes.insert("NumpadComma", NumpadComma);
        codes.insert("NumpadEnter", NumpadEnter);
        codes.insert("NumpadEqual", NumpadEquals);
        // OEM102,
        codes.insert("Period", Period);
        codes.insert("MediaPlayPause", PlayPause);
        codes.insert("Power", Power);
        codes.insert("MediaTrackPrevious", PrevTrack);
        codes.insert("AltRight", RAlt);
        codes.insert("BracketRight", RBracket);
        codes.insert("ControlRight", RControl);
        codes.insert("ShiftRight", RShift);
        codes.insert("MetaRight", RWin);
        codes.insert("OSRight", RWin);
        codes.insert("Semicolon", Semicolon);
        codes.insert("Slash", Slash);
        codes.insert("Sleep", Sleep);
        // Stop,
        codes.insert("NumpadSubtract", Subtract);
        // Sysrq,
        codes.insert("Tab", Tab);
        codes.insert("IntlRo", Underline);
        // Unlabeled,
        codes.insert("AudioVolumeDown", VolumeDown);
        codes.insert("AudioVolumeUp", VolumeUp);
        codes.insert("WakeUp", Wake);
        codes.insert("BrowserBack", WebBack);
        codes.insert("BrowserFavorites", WebFavorites);
        codes.insert("BrowserForward", WebForward);
        codes.insert("BrowserHome", WebHome);
        codes.insert("BrowserRefresh", WebRefresh);
        codes.insert("BrowserSearch", WebSearch);
        codes.insert("BrowserStop", WebStop);
        codes.insert("IntlYen", Yen);
        codes.insert("Copy", Copy);
        codes.insert("Paste", Paste);
        codes.insert("Cut", Cut);
        codes
    };
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum KeyCode {
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    Key0,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Escape,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    Snapshot,
    Scroll,
    Pause,
    Insert,
    Home,
    Delete,
    End,
    PageDown,
    PageUp,
    Left,
    Up,
    Right,
    Down,
    Back,
    Return,
    Space,
    Compose,
    Caret,
    Numlock,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    AbntC1,
    AbntC2,
    Add,
    Apostrophe,
    Apps,
    At,
    Ax,
    Backslash,
    Calculator,
    Capital,
    Colon,
    Comma,
    Convert,
    Decimal,
    Divide,
    Equals,
    Grave,
    Kana,
    Kanji,
    LAlt,
    LBracket,
    LControl,
    LShift,
    LWin,
    Mail,
    MediaSelect,
    MediaStop,
    Minus,
    Multiply,
    Mute,
    MyComputer,
    NavigateForward,
    NavigateBackward,
    NextTrack,
    NoConvert,
    NumpadComma,
    NumpadEnter,
    NumpadEquals,
    OEM102,
    Period,
    PlayPause,
    Power,
    PrevTrack,
    RAlt,
    RBracket,
    RControl,
    RShift,
    RWin,
    Semicolon,
    Slash,
    Sleep,
    Stop,
    Subtract,
    Sysrq,
    Tab,
    Underline,
    Unlabeled,
    VolumeDown,
    VolumeUp,
    Wake,
    WebBack,
    WebFavorites,
    WebForward,
    WebHome,
    WebRefresh,
    WebSearch,
    WebStop,
    Yen,
    Copy,
    Paste,
    Cut,
}

impl<S: AsRef<str>> From<S> for KeyCode {
    fn from(code: S) -> KeyCode {
        match CODES.get(code.as_ref()) {
            Some(code) => *code,
            None => KeyCode::Unlabeled
        }
    }
}

impl KeyCode {
    pub(crate) fn prevent_default(self) -> bool {
        use KeyCode::*;
        match self {
            Space | PageUp | PageDown | Escape => true,
            _ => false,
        }
    }
}
