package tui

import (
	"context"
	"fmt"
	"os"
	"time"

	"github.com/atotto/clipboard"
	"github.com/charmbracelet/bubbles/key"
	"github.com/charmbracelet/bubbles/list"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"

	v1 "github.com/skkil/clip/proto/v1"

	"github.com/skkil/clip/cli/internal/services"
	"github.com/skkil/clip/cli/internal/tui/panes"
)

type model struct {
	clipService *services.ClipService

	list    list.Model
	preview panes.Preview

	width, height int
	err           error
	status        string
}

func initialModel() model {
	return model{
		clipService: services.NewClipService(),
		list:        panes.NewList(),
		preview:     panes.NewPreview(),
	}
}

func (m model) Init() tea.Cmd {
	return m.fetchClips
}

func (m model) fetchClips() tea.Msg {
	clips, err := m.clipService.List(context.Background())
	if err != nil {
		return errMsg{err: err}
	}
	return clipsLoadedMsg{clips: clips}
}

// copySelected copies the currently selected clip's content to the system
// clipboard.
func (m model) copySelected() tea.Cmd {
	clip, ok := m.selectedClip()
	if !ok {
		return nil
	}
	return func() tea.Msg {
		return copiedMsg{err: clipboard.WriteAll(clip.Content)}
	}
}

func clearStatusAfter(d time.Duration) tea.Cmd {
	return tea.Tick(d, func(time.Time) tea.Msg {
		return statusClearMsg{}
	})
}

// selectedClip returns the clip currently highlighted in the list, if any.
func (m model) selectedClip() (*v1.Clip, bool) {
	item, ok := m.list.SelectedItem().(panes.ClipItem)
	if !ok {
		return nil, false
	}
	return item.Clip, true
}

// syncPreview refreshes the right pane's content to match the currently
// selected list item.
func (m *model) syncPreview() {
	clip, ok := m.selectedClip()
	if !ok {
		m.preview.SetContent("")
		return
	}
	m.preview.SetContent(clip.Content)
}

// layout returns the left (list) and right (preview) pane content widths and
// the shared pane height, after accounting for each pane's border.
func (m model) layout() (leftW, rightW, paneH int) {
	frameW, frameH := paneStyle.GetFrameSize()

	total := m.width
	left := total / 2
	right := total - left

	return left - frameW, right - frameW, m.height - frameH
}

func (m model) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.WindowSizeMsg:
		m.width = msg.Width
		m.height = msg.Height

		leftW, rightW, paneH := m.layout()
		m.list.SetSize(leftW, paneH)
		m.preview.SetSize(rightW, paneH)
		m.syncPreview()
		return m, nil

	case tea.KeyMsg:
		if !m.list.SettingFilter() {
			switch {
			case key.Matches(msg, keys.Quit):
				return m, tea.Quit
			case key.Matches(msg, keys.Refresh):
				return m, m.fetchClips
			case key.Matches(msg, keys.Copy):
				return m, m.copySelected()
			}
		}

	case copiedMsg:
		if msg.err != nil {
			m.status = fmt.Sprintf("failed to copy: %v", msg.err)
		} else {
			m.status = "copied to clipboard"
		}
		return m, clearStatusAfter(statusDisplayDuration)

	case statusClearMsg:
		m.status = ""
		return m, nil

	case clipsLoadedMsg:
		m.err = nil
		cmd := m.list.SetItems(panes.Items(msg.clips))
		m.syncPreview()
		return m, cmd

	case errMsg:
		m.err = msg.err
		return m, nil
	}

	var cmd tea.Cmd
	m.list, cmd = m.list.Update(msg)
	m.syncPreview()
	return m, cmd
}

func (m model) View() string {
	if m.err != nil {
		return errStyle.Render(fmt.Sprintf("failed to load clips: %v\n\npress r to retry, q to quit", m.err))
	}

	left := paneStyle.Render(m.list.View())
	right := m.preview.View(m.status)

	return lipgloss.JoinHorizontal(lipgloss.Top, left, right)
}

func Run() error {
	p := tea.NewProgram(initialModel(), tea.WithAltScreen())
	if _, err := p.Run(); err != nil {
		fmt.Fprintf(os.Stderr, "error running TUI: %v\n", err)
		return err
	}
	return nil
}
