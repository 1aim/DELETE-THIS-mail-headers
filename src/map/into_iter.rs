use std::vec;

use common::encoder::EncodableInHeader;
use ::HeaderName;

use super::HeaderMap;

impl IntoIterator for HeaderMap {

    type Item = (HeaderName, Box<EncodableInHeader>);
    type IntoIter = vec::IntoIter<(HeaderName, Box<EncodableInHeader>)>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner_map.into_iter()
    }
}