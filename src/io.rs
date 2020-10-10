// use std::collections::HashMap;
//
// #[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Copy, Clone)]
// pub enum FlvSeekFrom {
//     Header,
//     MetaData,
//     PreTagSize(i64),
//     Tag(i64),
// }
//
// pub trait IndexCache {
//     fn get(&self, seek_from: FlvSeekFrom) -> Option<u64>;
//     fn put(&mut self, seek_from: FlvSeekFrom, offset: u64);
// }
//
// pub struct FlvIndexCache {
//     cache: HashMap<FlvSeekFrom, u64>,
// }
//
// impl FlvIndexCache {
//     pub fn new() -> Self {
//         Self {
//             cache: HashMap::new(),
//         }
//     }
//
//     pub fn with_capacity(capacity: usize) -> Self {
//         Self {
//             cache: HashMap::with_capacity(capacity),
//         }
//     }
// }
//
// impl IndexCache for FlvIndexCache {
//     fn get(&self, seek_from: FlvSeekFrom) -> Option<u64> {
//         self.cache.get(&seek_from).cloned()
//     }
//
//     fn put(&mut self, seek_from: FlvSeekFrom, offset: u64) {
//         self.cache.insert(seek_from, offset);
//     }
// }
//
// impl IndexCache for () {
//     fn get(&self, _: FlvSeekFrom) -> Option<u64> {
//         None
//     }
//
//     fn put(&mut self, _: FlvSeekFrom, _: u64) {}
// }
