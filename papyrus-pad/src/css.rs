/// CSS for all pad items.
pub const PAD_CSS: &'static str = r##"
.ansi-renderer-line {
	flex-direction: row;
}

.ansi-renderer-text {
	color: [[ ansi_esc_color | white ]];
	text-align: left;
	line-height: 135%;
	font-size: 1em;
	font-family: Lucida Console,Lucida Sans Typewriter,monaco,Bitstream Vera Sans Mono,monospace;
}

.ansi-renderer-text:hover {
	border: 1px solid #9b9b9b;
}

.repl-terminal {
	background-color: black;
	padding: 5px;
}

#completion-prompt {
	background-color: white;
}
"##;
