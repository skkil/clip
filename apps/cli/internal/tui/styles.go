package tui

import (
	"time"

	"github.com/charmbracelet/lipgloss"
)

const statusDisplayDuration = 1500 * time.Millisecond

var (
	errStyle = lipgloss.NewStyle().Foreground(lipgloss.Color("9"))

	paneStyle = lipgloss.NewStyle().
			Border(lipgloss.RoundedBorder()).
			BorderForeground(lipgloss.Color("240"))
)
