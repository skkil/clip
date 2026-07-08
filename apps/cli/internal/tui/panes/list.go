package panes

import (
	"github.com/charmbracelet/bubbles/list"
	v1 "github.com/skkil/clip/proto/v1"
)

const previewLineLimit = 80

// ClipItem renders as a single line in the list: the clip content, truncated
// to fit, with no secondary description line.
type ClipItem struct {
	Clip *v1.Clip
}

func (i ClipItem) Title() string {
	content := oneLine(i.Clip.Content)
	if len(content) > previewLineLimit {
		content = content[:previewLineLimit] + "…"
	}
	return content
}

func (i ClipItem) Description() string { return "" }

func (i ClipItem) FilterValue() string { return i.Clip.Content }

// oneLine collapses a multi-line clip into a single line for the list preview.
func oneLine(s string) string {
	out := make([]rune, 0, len(s))
	prevSpace := false
	for _, r := range s {
		if r == '\n' || r == '\r' || r == '\t' {
			r = ' '
		}
		if r == ' ' {
			if prevSpace {
				continue
			}
			prevSpace = true
		} else {
			prevSpace = false
		}
		out = append(out, r)
	}
	return string(out)
}

// singleLineDelegate is a list.ItemDelegate that renders each entry on one
// line (no wrapped description line beneath it).
type singleLineDelegate struct {
	list.DefaultDelegate
}

func newSingleLineDelegate() singleLineDelegate {
	d := list.NewDefaultDelegate()
	d.ShowDescription = false
	d.SetSpacing(0)
	return singleLineDelegate{DefaultDelegate: d}
}

// NewList builds the list.Model used for the left pane.
func NewList() list.Model {
	l := list.New(nil, newSingleLineDelegate(), 0, 0)
	l.Title = "Clips"
	l.SetShowHelp(true)
	return l
}

// Items converts clips into list.Item values for list.Model.SetItems.
func Items(clips []*v1.Clip) []list.Item {
	items := make([]list.Item, len(clips))
	for i, clip := range clips {
		items[i] = ClipItem{Clip: clip}
	}
	return items
}
