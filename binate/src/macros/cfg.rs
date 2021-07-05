macro_rules! feature {
    (
        #![$meta:meta]
        $($item:item)*
    ) => {
        $(
            #[cfg($meta)]
            #[cfg_attr(docsrs, doc(cfg($meta)))]
            $item
        )*
    }
}

macro_rules! cfg_loom {
    ($($item:item)*) => {
        $(
            #[cfg(loom)]
            $item
        )*
    }
}

macro_rules! cfg_not_loom {
    ($($item:item)*) => {
        $(
            #[cfg(not(loom))]
            $item
        )*
    }
}

macro_rules! cfg_frame {
    ($($item:item)*) => {
        $(
            #[cfg(feature = "frame")]
            #[cfg_attr(docsrs, doc(cfg(feature = "frame")))]
            $item
        )*
    }
}

macro_rules! cfg_not_frame {
    ($($item:item)*) => {
        $(
            #[cfg(not(feature = "frame"))]
            $item
        )*
    }
}
