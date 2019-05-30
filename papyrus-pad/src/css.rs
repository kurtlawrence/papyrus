/// CSS for all pad items.
pub const PAD_CSS: &'static str = r##"
.repl-terminal {
	background-color: black;
	padding: 5px;
}

.repl-terminal-line {
	flex-direction: row;
}

.repl-terminal-text {
	color: [[ ansi_esc_color | white ]];
	text-align: left;
	line-height: 135%;
	font-size: 1em;
	font-family: Lucida Console,Lucida Sans Typewriter,monaco,Bitstream Vera Sans Mono,monospace;
}

.repl-terminal-text:hover {
	border: 1px solid #9b9b9b;
}

#completion-prompt {
	background-color: white;
}
"##;
