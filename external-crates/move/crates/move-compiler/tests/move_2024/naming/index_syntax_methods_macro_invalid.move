module a::invalid0 {

    public struct S<T> has drop { t: vector<T> }

    #[allow(unused_variable)]
    #[syntax(index)]
    public macro fun lookup_t<$T>($s: &S<$T>, $i: u64): &mut u64 { abort 0 }

}

module a::invalid1 {

    public struct S<T> has drop { t: vector<T> }

    #[allow(unused_variable)]
    #[syntax(index)]
    public macro fun lookup_t<$T>($s: &mut S<$T>, $i: u64): &u64 { abort 0 }

}

module a::invalid2 {

    public struct S<T> has drop { t: vector<T> }

    #[allow(unused_variable)]
    #[syntax(index)]
    public macro fun lookup_t<$T>($s: &mut S<$T>, $i: u64): &u64 { abort 0 }

    #[allow(unused_variable)]
    #[syntax(index)]
    public macro fun lookup_mut_t<$T>($s: &mut S<$T>, $i: u64): &u64 { abort 0 }

}
