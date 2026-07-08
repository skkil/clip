package tui

import v1 "github.com/skkil/clip/proto/v1"

type clipsLoadedMsg struct {
	clips []*v1.Clip
}

type errMsg struct {
	err error
}

type copiedMsg struct {
	err error
}

type statusClearMsg struct{}
