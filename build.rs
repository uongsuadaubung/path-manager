fn main() {
    // Chỉ chạy khi build cho target Windows
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let mut res = winres::WindowsResource::new();
        res.set_icon("icon.ico");
        
        // Thử biên dịch icon, nếu lỗi (do thiếu rc.exe trên Linux) thì chỉ in ra cảnh báo, không làm dừng quá trình build
        if let Err(e) = res.compile() {
            eprintln!("Warning: Resource compilation failed: {}", e);
        }
    }
}
