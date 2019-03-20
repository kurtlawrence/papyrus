use libloading::{Library, Symbol};
use std::io::{self, Write};
use std::path::Path;

/// We always send through an immutable reference.
/// The function signature will **_always_** be `(app_data: &D) -> String`.
type DataFunc<D> = unsafe fn(&D) -> String;

type ExecResult = Result<String, &'static str>;

pub fn exec<'c, P, Data>(library_file: P, function_name: &str, app_data: &Data) -> ExecResult
where
    P: AsRef<Path>,
{
    let lib = get_lib(library_file)?;
    let func = get_func(&lib, function_name)?;

    let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe { func(app_data) }));

    match res {
        Ok(s) => Ok(s),
        Err(_) => Err("a panic occured with evaluation"),
    }
}

pub fn exec_and_redirect<'c, P: AsRef<Path>, Data, W: Write + Send>(
    library_file: P,
    function_name: &str,
    app_data: &Data,
    mut output_wtr: W,
) -> ExecResult {
    let lib = get_lib(library_file)?;
    let func = get_func(&lib, function_name)?;

    let (tx, rx) = crossbeam::channel::bounded(0);

    let (stdout_gag, stderr_gag) =
        get_gags().map_err(|_| "failed to apply redirect gags on stdout and stderr")?;

    let res = crossbeam::scope(|scope| {
        let jh = scope.spawn(|_| redirect_output_loop(&mut output_wtr, rx, stdout_gag, stderr_gag));

        let res =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe { func(app_data) }));

        tx.send(()).expect("sending signal to stop gagging failed");
        jh.join()
            .expect("joining gagging thread failed")
            .expect("failed to redirect output loop");
        res
    });

    let res = res.map_err(|_| "crossbeam scoping failed")?;

    match res {
        Ok(s) => Ok(s),
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
    Library::new(path.as_ref()).map_err(|_| "failed to load library file")
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

fn get_gags() -> io::Result<(shh::ShhStdout, shh::ShhStderr)> {
    Ok((shh::stdout()?, shh::stderr()?))
}

fn redirect_output_loop<W: Write, R1: io::Read, R2: io::Read>(
    wtr: &mut W,
    rx: crossbeam::channel::Receiver<()>,
    mut stdout_gag: R1,
    mut stderr_gag: R2,
) -> io::Result<()> {
    loop {
        std::thread::sleep(std::time::Duration::from_millis(2)); // add in some delay as reading occurs to avoid smashing cpu.

        let mut buf = Vec::new();

        // read/write stderr first
        stderr_gag.read_to_end(&mut buf)?;

        stdout_gag.read_to_end(&mut buf)?;

        wtr.write_all(&buf)?;

        match rx.try_recv() {
            Ok(_) => break,                                               // stop signal sent
            Err(crossbeam::channel::TryRecvError::Disconnected) => break, // tx dropped
            _ => (),
        };
    }

    Ok(())
}
