/// CSS for all pad items.
pub const PAD_CSS: &'static str = r##"
.ansi-renderer-line {
	flex-direction: row;
	height: 25px;
}

.ansi-renderer-text {
	color: [[ ansi_esc_color | !z0-fg-colour ]];
}

.repl-terminal {
	position: relative;
	background-color: !z0-bg-colour;
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

	color: !z0-fg-colour;	
	background-color: !z2-bg-colour;
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
	background: !z3-fg-colour;
}

.completion-prompt-item-type {
	text-align: right;
	color: !z2-fg-colour;
}

.completion-prompt-info {
	position: absolute;
	top: [[ top | 0px ]];
	left: 200px;

	font-size: 8pt;
}
"##;
