use klover::{class_loader::bootstrap_cld::BootstrapCLD, interpreter::interpreter::{Interpreter, InvokeOutcome, ReturnValue}, runtime::{arguments::Arguments, vm::vm_init}};

const TEST_CLASS_PATH: &str = "../test_data/classes";

#[test]
fn test_loading() {
    let args = Arguments {
        bs_class_path: TEST_CLASS_PATH.into(),
        xmx: 64 * 1024 * 1024
    };

    vm_init(args);

    let klass = BootstrapCLD::find_class("SimpleAddition").unwrap();
    let obj = BootstrapCLD::find_class("java/lang/Object").unwrap().as_normal().unwrap();
    // println!("{:?}, {:?}", klass.this_klass.name.utf8(), obj.this_klass.name.utf8());

    // let method = klass.find_method(&"add".into(), &"(II)I".into());
    let m_name = "add".into();
    let m_desc = "(II)I".into();

    let m = klass.as_normal().unwrap().find_method(&m_name, &m_desc).unwrap();

    let mut intp = Interpreter::new(4096);
    let i_args = vec![1, 2];
    let res = intp.invoke_static(klass.as_normal().unwrap(), m, &i_args).unwrap();

    match res {
        InvokeOutcome::Returned(Some(ReturnValue::Int(x))) => println!("{}", x),
        _ => panic!()
    }
}
