#[cfg(test)]
mod test_knet {
    use derive_knet::DeriveKnet;
    use knet::network::trait_knet::KnetTransform;

    #[derive(Debug, PartialEq, Default, Copy, Clone)]
    struct Foo {
        x: i32,
        y: i32,
    }

    #[derive(DeriveKnet, Debug, PartialEq)]
    enum Data {
        Byte(u8),
        Integer(i32),
        Char(char),
        FooStruct(Foo),
    }

    #[test]
    fn test_with_structure() {
        let data = Data::FooStruct(Foo::default());
        let raw = data.serialize();
        assert_eq!(Data::from_raw(&raw), data);
    }

    #[test]
    fn test_with_structure_change_variable() {
        let mut foo = Foo::default();
        foo.x = 42;
        foo.y = 21;
        let data = Data::FooStruct(foo);
        let raw = data.serialize();
        assert_eq!(Data::from_raw(&raw), data);
    }

    #[test]
    fn serialize_deserialize() {
        let data_integer = Data::Integer(8i32);
        let mut data_char = Data::Char('c');
        let integer_serialize = data_integer.serialize();
        data_char.deserialize(&integer_serialize);
        assert_eq!(data_integer, data_char);
        assert_eq!(Data::Integer(8i32), data_char);
        assert_eq!(Data::Integer(8i32), data_integer);
    }

    #[test]
    fn print_serialize() {
        let data8 = Data::Byte(8u8);
        let data16 = Data::Byte(16u8);
        let data32 = Data::Byte(32u8);
        assert_eq!([66, 121, 116, 101, 0, 0, 0, 0, 0, 8], data8.serialize()[..]);
        assert_eq!(
            [66, 121, 116, 101, 0, 0, 0, 0, 0, 16],
            data16.serialize()[..]
        );
        assert_eq!(
            [66, 121, 116, 101, 0, 0, 0, 0, 0, 32],
            data32.serialize()[..]
        );
    }

    #[test]
    fn compare_to_bytes() {
        let data8 = Data::Byte(8u8);
        let data16 = Data::Byte(16u8);
        let data32 = Data::Byte(32u8);
        assert_eq!([66, 121, 116, 101, 0, 0, 0, 0, 0, 8], data8.serialize()[..]);
        assert_eq!(
            [66, 121, 116, 101, 0, 0, 0, 0, 0, 16],
            data16.serialize()[..]
        );
        assert_eq!(
            [66, 121, 116, 101, 0, 0, 0, 0, 0, 32],
            data32.serialize()[..]
        );
    }

    #[test]
    fn from_raw() {
        let data = Data::from_raw(&[66, 121, 116, 101, 0, 0, 0, 0, 0, 42].to_vec());
        assert_eq!(Data::Byte(42_u8), data);
    }
}
