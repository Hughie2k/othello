fn main() {
    // if std::env::consts::OS == "windows" {
    //     unsafe {
    //         use windows::Win32::System::Console::*;
    //         let stdout = GetStdHandle(STD_OUTPUT_HANDLE).unwrap();
    //         let stderr = GetStdHandle(STD_ERROR_HANDLE).unwrap();
    //         let mut mode = CONSOLE_MODE(0);
    //         GetConsoleMode(stdout, &mut mode);
    //         mode |= ENABLE_VIRTUAL_TERMINAL_PROCESSING;
    //         SetConsoleMode(stdout, mode);
    //         GetConsoleMode(stderr, &mut mode);
    //         mode |= ENABLE_VIRTUAL_TERMINAL_PROCESSING;
    //         SetConsoleMode(stderr, mode);
    //     }
    // }

    //othello::gui::run();
}
