package main

import (
	osapi "github.com/Bensterriblescripts/Lib-Handlers/os"
)

type HWNDManager struct{}

var activeWindows []osapi.Window

func (h *HWNDManager) GetAllActiveWindows() []osapi.Window {
	activeWindows = osapi.GetActiveWindows()
	return activeWindows
}
