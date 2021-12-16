macro_rules! cfg {
    (
        #[$meta:meta]
        $($item:item)*
    ) => {
        $(
            #[cfg($meta)]
            #[cfg_attr(docsrs, doc(cfg($meta)))]
            $item
        )*
    }
}

macro_rules! cfg_not {
    (
        #[$meta:meta]
        $($item:item)*
    ) => {
        $(
            #[cfg(not($meta))]
            $item
        )*
    }
}
