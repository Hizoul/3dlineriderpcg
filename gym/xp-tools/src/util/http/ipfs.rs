
pub fn ipfs_url_cat(ipfs_base_url: &str, ipfs_cid: &str) -> String {
    format!("{}/api/v0/cat?arg={}", ipfs_base_url, ipfs_cid)
}