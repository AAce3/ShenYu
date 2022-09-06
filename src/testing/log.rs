use std::fs::OpenOptions;

pub const DEFAULT_LOG: &str = "log.txt";

#[macro_export]
macro_rules! log_data {
    ($board:ident, $( $name:literal, $item:ident)+) => {
        use $crate::testing::log::DEFAULT_LOG;
        use std::io::Write;
        use std::fs::OpenOptions;
        let time = chrono::Local::now();

        let mut file = OpenOptions::new().write(true).open(DEFAULT_LOG).unwrap();
        write!(file, "[{}]", time.format("%Y-%m-%d %H:%M:%S")).unwrap();
        write!(file, " [Fen: {}]", $board.generate_fen()).unwrap();
        $(write!(file, " [{}: {:?}]", $name, $item).unwrap();)+
        write!(file, "\n").unwrap();
    };

    ($board:ident, $path:literal, $($name:literal, $item:ident)+) => {
        use std::io::Write;
        use std::fs::OpenOptions;
        let time = chrono::Local::now();

        let mut file = OpenOptions::new().write(true).open($path).unwrap();
        write!(file, "[{}]", time.format("%Y-%m-%d %H:%M:%S")).unwrap();
        write!(file, " [Fen: {}]", $board.generate_fen()).unwrap();
        $(write!(file, " [{}: {:?}]", $name, $item).unwrap();)+
        write!(file, "\n").unwrap();
    };
}

pub fn clear_logs(){
    OpenOptions::new().write(true).create(true).truncate(true).open(DEFAULT_LOG).unwrap();
}