/// CSS for all pad items.
pub const PAD_CSS: &'static str = r##"
.ansi-renderer-line {
	flex-direction: row;
}

.ansi-renderer-text {
	color: [[ ansi_esc_color | white ]];
	line-height: 135%;
}

.ansi-renderer-text:hover {
	border: 1px solid #9b9b9b;
}

.repl-terminal {
	background-color: black;
	padding: 5px;
	text-align: left;
	font-size: 1em;
	font-family: Lucida Console,Lucida Sans Typewriter,monaco,Bitstream Vera Sans Mono,monospace;
}

.completion-prompt {
	position: absolute;
	width: 500px;
	left: [[ left | 0px ]];
	top: [[ top | 0px ]];

	background-color: white;

}

.completion-prompt-item {
	height: [[ height | auto ]];
	width: 200px;

	flex-direction: row;
}

.completion-prompt-item:hover {
	background: #9b9b9b;
}

.completion-prompt-item-kb {
	background: blue;
}

.completion-prompt-info {
	position: absolute;
	top: [[ top | 0px ]];
	left: 200px;

	font-size: 8pt;
	word-wrap: normal;
}
"##;
