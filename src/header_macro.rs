pub use soft_ascii_string::{ SoftAsciiStr as _SoftAsciiStr };


/// Defines a new header types with given type name, filed name and component
///
/// Note that the name is not checked/validated, it has to be ascii, a valid
/// header field name AND has to comply with the naming schema (each word
/// seperated by `'-'` starts with a capital letter and no cappital letter
/// follow, e.g. "Message-Id" is ok but "Message-ID" isn't).
///
/// This macro will create a test which will check if the used field names
/// are actually valid and appears only once (_per def_header macro call_)
/// so as long as test's are run any invalid name will be found.
///
/// Note that even if a invalid name was used and test where ignored/not run
/// this will _not_ cause an rust safety issue, but can still cause bugs under
/// some circumstances (e.g. if you have multiple differing definitions of the
/// same header with different spelling (at last one failed the test) like e.g.
/// when you override default implementations of fields).
///
/// The macros expects following items:
///
/// 1. `test_name`, which is the name the auto-generated test will have
/// 2. `scope`, the scope all components are used with, this helps with some
///    name collisions. Use `self` to use the current scope.
/// 3. a list of header definitions consisting of:
///
///    1. `<typename>` the name the type of the header will have, i.e. the name of a zero-sized
///       struct which will be generated
///    2. `<qunatity>`, stating weather the header can appear at most one time (`maxOne`)
///       or any number of times (`anyNumber`). Note: that only `Date` and `From` are
///       required headers, no other can be made into such.
///    3. `unchecked` a hint to make people read the documentation and not forget the the
///       folowing data is `unchecked` / only vaidated in the auto-generated test
///    4. `"<header_name>"` the header name in a syntax using `'-'` to serperate words,
///       also each word has to start with a capital letter and be followed by lowercase
///       letters additionaly to being a valid header field name. E.g. "Message-Id" is
///       ok, but "Message-ID" is not. (Note that header field name are on itself ignore
///       case, but by enforcing a specific case in the encoder equality checks can be
///       done on byte level, which is especially usefull for e.g. placing them as keys
///       into a HashMap or for performance reasons.
///    5. `<component>` the name of the type to use ing `scope` a the component type of
///       the header. E.g. `Unstructured` for an unstructured header field (which still
///       support Utf8 through encoded words)
///    6. `None`/`<ident>`, None or the name of a validator function (if there is one).
///       This function is called before encoding with the header map as argument, and
///       can cause a error. Use this to enfore contextual limitations like having a
///       `From` with multiple mailboxes makes `Sender` an required field.
///
/// # Example
///
/// ```norun
/// def_headers! {
///     // the name of the auto-generated test
///     test_name: validate_header_names,
///     // the scope from which all components should be imported
///     // E.g. `DateTime` refers to `components::DateTime`.
///     scope: components,
///     // definitions of the headers or the form
///     // <type_name>, <quantitiy>, unchecked { <struct_name> }, <component>, <contextual_validator>
///     Date,     maxOne,    unchecked { "Date"          },  DateTime,       None,
///     From,     maxOne,    unchecked { "From"          },  MailboxList,    validator_from,
///     Subject,  maxOne,    unchecked { "Subject"       },  Unstructured,   None,
///     Comments, anyNumber, unchecked { "Comments"      },  Unstructured,   None,
/// }
/// ```
#[macro_export]
macro_rules! def_headers {
    (
        test_name: $tn:ident,
        scope: $scope:ident,
        $(
            $(#[$attr:meta])*
            $name:ident, $multi:ident, unchecked { $hname:tt }, $component:ident, $validator:ident
        ),+
    ) => (
        $(
            $(#[$attr])*
            pub struct $name;

            impl $crate::Header for  $name {
                const MAX_COUNT_EQ_1: bool = def_headers!(_PRIV_boolify $multi);
                type Component = $scope::$component;

                fn name() -> $crate::HeaderName {
                    let as_str: &'static str = $hname;
                    $crate::HeaderName::from_ascii_unchecked( as_str )
                }

                const CONTEXTUAL_VALIDATOR:
                    Option<
                        fn(&$crate::map::HeaderMap)
                        -> Result<(), $crate::error::HeaderValidationError>
                    > =
                        def_headers!{ _PRIV_mk_validator $validator };
            }
        )+

        $(
            def_headers!{ _PRIV_impl_marker $multi $name }
        )+

        //TODO warn if header type name and header name diverges
        // (by stringifying the type name and then ziping the
        //  array of type names with header names removing
        //  "-" from the header names and comparing them to
        //  type names)


        #[cfg(test)]
        const HEADER_NAMES: &[ &str ] = &[ $(
            $hname
        ),+ ];

        #[test]
        fn $tn() {
            use std::collections::HashSet;
            use $crate::__common::encoder::EncodableInHeader;

            let mut name_set = HashSet::new();
            for name in HEADER_NAMES {
                if !name_set.insert(name) {
                    panic!("name appears more than one time in same def_headers macro: {:?}", name);
                }
            }
            fn can_be_trait_object<EN: EncodableInHeader>( v: Option<&EN> ) {
                let _ = v.map( |en| en as &EncodableInHeader );
            }
            $(
                can_be_trait_object::<$scope::$component>( None );
            )+
            for name in HEADER_NAMES {
                let res = $crate::HeaderName::new(
                    $crate::soft_ascii_string::SoftAsciiStr::from_str(name).unwrap()
                );
                if res.is_err() {
                    panic!( "invalid header name: {:?} ({:?})", name, res.unwrap_err() );
                }
            }
        }
    );
    (_PRIV_mk_validator None) => ({ None });
    (_PRIV_mk_validator $validator:ident) => ({ Some($validator) });
    (_PRIV_boolify anyNumber) => ({ false });
    (_PRIV_boolify maxOne) => ({ true });
    (_PRIV_boolify $other:tt) => (
        compile_error!( "only `maxOne` or `anyNumber` are valid" )
    );
    ( _PRIV_impl_marker anyNumber $name:ident ) => (
        //do nothing here
    );
    ( _PRIV_impl_marker maxOne $name:ident ) => (
        impl $crate::SingularHeaderMarker for $name {}
    );
}
