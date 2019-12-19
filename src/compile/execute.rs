use ::kserd::Kserd;
use libloading::{Library, Symbol};
use std::path::Path;

/// We don't type anything here. You must be **VERY** careful to pass through the correct borrow to match the
/// function signature!
type DataFunc<D> = unsafe fn(D) -> Kserd<'static>;

type ExecResult = Result<(Kserd<'static>, Library), &'static str>;

pub(crate) fn exec<P: AsRef<Path>, D>(
    library_file: P,
    function_name: &str,
    app_data: D,
) -> ExecResult {
    exec_no_redirect(library_file, function_name, app_data)
}

fn exec_no_redirect<P: AsRef<Path>, Data>(
    library_file: P,
    function_name: &str,
    app_data: Data,
) -> ExecResult {
    let lib = get_lib(library_file)?;
    let func = get_func(&lib, function_name)?;

    let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe { func(app_data) }));

    match res {
        Ok(kserd) => Ok((kserd, lib)),
        Err(_) => Err("a panic occured with evaluation"),
    }
}

fn get_lib<P: AsRef<Path>>(path: P) -> Result<Library, &'static str> {
    // If segfaults are occurring maybe use this, SIGSEV?
    // This is shown in https://github.com/nagisa/rust_libloading/issues/41
    // let lib: Library =
    // 	libloading::os::unix::Library::open(Some(library_file.as_ref()), 0x2 | 0x1000)
    // 		.unwrap()
    // 		.into();
    Library::new(path.as_ref()).map_err(|e| {
        error!("failed to load library file: {}", e);
        "failed to load library file"
    })
}

fn get_func<'l, Data>(
    lib: &'l Library,
    name: &str,
) -> Result<Symbol<'l, DataFunc<Data>>, &'static str> {
    unsafe {
        lib.get(name.as_bytes())
            .map_err(|_| "failed to find function in library")
    }
}
