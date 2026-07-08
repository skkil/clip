package panes

import (
	"github.com/charmbracelet/bubbles/viewport"
	"github.com/charmbracelet/lipgloss"
)

const previewTitleHeight = 1

var (
	previewTitleStyle = lipgloss.NewStyle().
				Bold(true).
				Padding(0, 1)

	statusStyle = lipgloss.NewStyle().
			Foreground(lipgloss.Color("42")).
			Padding(0, 1)

	focusedPaneStyle = lipgloss.NewStyle().
				Border(lipgloss.RoundedBorder()).
				BorderForeground(lipgloss.Color("62"))
)

// Preview renders the right-hand pane: a title bar (with an optional
// transient status message) above a scrollable content viewport.
type Preview struct {
	title    string
	viewport viewport.Model
}

func NewPreview() Preview {
	return Preview{
		title:    "Preview",
		viewport: viewport.New(0, 0),
	}
}

// SetSize sets the content width/height, accounting for the title row.
func (p *Preview) SetSize(width, height int) {
	p.viewport.Width = width
	p.viewport.Height = height - previewTitleHeight
}

// SetContent replaces the previewed clip and scrolls back to the top.
func (p *Preview) SetContent(content string) {
	p.viewport.SetContent(content)
	p.viewport.GotoTop()
}

// View renders the pane, appending status (if any) to the title row.
func (p Preview) View(status string) string {
	title := previewTitleStyle.Render(p.title)
	if status != "" {
		title = lipgloss.JoinHorizontal(lipgloss.Top, title, statusStyle.Render(status))
	}
	return focusedPaneStyle.Render(lipgloss.JoinVertical(lipgloss.Left, title, p.viewport.View()))
}
