macro_rules! cb {
	($type:ident, $fn:ident) => {
			fn $fn(
			data: &StackCheckedPointer<T>,
			app_state_no_data: &mut AppStateNoData<T>,
			window_event: &mut CallbackInfo<T>,
		) -> UpdateScreen {
			unsafe { data.invoke_mut($type::$fn, app_state_no_data, window_event) }
		}
	};
}

pub mod colour;
pub mod pad;

