# NOTES

## Direction

Currently I see the repl as quite static. Load data, run, interact but generally don't have much idea with compiling.

I would like to move to a more interactive environment, where larger data can be statically loaded and called in to give the repl a more dynamic feel, like the python examples. there is more research required to achieve this.

to assist however i think i need to focus on how the user interacts with the repl. it is no good just having everything cmd line based and not letting the user toy around with small programs. In this sense I want to draw on features of linqpad and vscode where you can set up a mini ide environment. i believe the major mode of editing will still be done through the shell, maybe with a system that can translate changes to the repl through some text editing interface. the main feature however will be the repl which is still interacted with using the shell. 

I must keep my goal well scoped and focused. what i want is a system that lets you play with a data set in close to real-time, and be able to interrogate that data. Think of features of craigs dataflows, linqpad, juypter, clojure, etc. It would be easy to fall into the trap of just creating an ide, to differentiate I need to focus on data manipulation, editing statements, and data visualisation.

I think this work will be suited to a larger practical project so I can develop as i go, i need to seriously consider the mining reconciliation tool.

ideas... lagged compilation?