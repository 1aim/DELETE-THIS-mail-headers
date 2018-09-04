use std::vec;

use ::HeaderName;
use ::header::HeaderObj;

use super::HeaderMap;

impl IntoIterator for HeaderMap {

    type Item = (HeaderName, Box<HeaderObj>);
    type IntoIter = vec::IntoIter<(HeaderName, Box<HeaderObj>)>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner_map.into_iter()
    }
}