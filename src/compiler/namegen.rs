
use small_uid::SmallUid;

use super::get_cstring;

pub fn gen_id() -> *mut i8 {
    let id = SmallUid::new().to_string();
    get_cstring(id)
}

pub fn gen_id_pre(pre: String) -> *mut i8 {
    let mut id = String::new();
    id.push_str(&pre);
    id.push('_');
    id.push_str(&SmallUid::new().to_string());
    get_cstring(id)
}

pub fn gen_id_post(post: String) -> *mut i8 {
    let mut id = String::new();
    id.push_str(&SmallUid::new().to_string());
    id.push('_');
    id.push_str(&post);
    get_cstring(id)
}

pub fn gen_id_prepost(pre: String, post: String) -> *mut i8 {
    let mut id = String::new();
    id.push_str(&pre);
    id.push('_');
    id.push_str(&SmallUid::new().to_string());
    id.push('_');
    id.push_str(&post);
    get_cstring(id)
}

