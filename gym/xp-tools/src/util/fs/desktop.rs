pub fn list_dir(dir_name: &str) -> Vec<String> {
  use std::fs::read_dir;
  read_dir(dir_name).unwrap()
    .map(|res| res.unwrap().path().as_path().to_str().unwrap().to_owned())
    .collect()
}

pub async fn read_file(path: &str) -> Vec<u8> {
  std::fs::read(path).unwrap()
}

pub fn create_dir_if_it_doesnt_exist(dir_name: &str) {
  if std::fs::metadata::<&str>(dir_name).is_err() {
    std::fs::create_dir::<&str>(dir_name).unwrap();
  }
}

const KV_SUBDIR: &str = "rs_kvstore";

pub fn kv_store_get(key: &str) -> String {
  let config_dir_opt = dirs::config_dir();
  if let Some(mut config_dir) = config_dir_opt {
    config_dir.push(KV_SUBDIR);
    config_dir.push(key);
    if config_dir.exists() && config_dir.is_file() {
      return String::from_utf8(std::fs::read(config_dir.to_str().unwrap()).unwrap()).unwrap();
    }
  }
  String::new()
}

pub fn kv_store_set(key: &str, value: &str) {
  let config_dir_opt = dirs::config_dir();
  if let Some(mut config_dir) = config_dir_opt {
    config_dir.push(KV_SUBDIR);
    create_dir_if_it_doesnt_exist(config_dir.to_str().unwrap());
    config_dir.push(key);
    std::fs::write(config_dir.to_str().unwrap(), value).unwrap();
  }
}