macro_rules! cfg_io_std {
    ($($item:item)*) => {
        $(
            #[cfg(feature = "io-std")]
            #[cfg_attr(docsrs, doc(cfg(feature = "io-std")))]
            $item
        )*
    }
}

macro_rules! cfg_io_tokio {
    ($($item:item)*) => {
        $(
            #[cfg(feature = "io-tokio")]
            #[cfg_attr(docsrs, doc(cfg(feature = "io-tokio")))]
            $item
        )*
    }
}
