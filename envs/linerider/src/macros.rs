#[macro_export]
macro_rules! try_read {
    ($obj:expr) => {{
      let mut attempt = $obj.try_read();
      while attempt.is_err() {
        std::thread::sleep(std::time::Duration::from_nanos(10));
        attempt = $obj.try_read();
      }
      attempt.unwrap()}
    };
}

#[macro_export]
macro_rules! try_write {
    ($obj:expr) => {{
      let mut attempt = $obj.try_write();
      while attempt.is_err() {
        std::thread::sleep(std::time::Duration::from_nanos(10));
        attempt = $obj.try_write();
      }
      attempt.unwrap()}
    };
}