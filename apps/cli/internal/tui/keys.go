package tui

import "github.com/charmbracelet/bubbles/key"

// keyMap defines the global (non-filtering) key bindings for the app.
type keyMap struct {
	Quit    key.Binding
	Refresh key.Binding
	Copy    key.Binding
}

var keys = keyMap{
	Quit: key.NewBinding(
		key.WithKeys("q", "ctrl+c"),
		key.WithHelp("q", "quit"),
	),
	Refresh: key.NewBinding(
		key.WithKeys("r"),
		key.WithHelp("r", "refresh"),
	),
	Copy: key.NewBinding(
		key.WithKeys("enter"),
		key.WithHelp("enter", "copy"),
	),
}
