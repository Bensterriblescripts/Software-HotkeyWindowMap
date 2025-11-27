package main

import (
	"github.com/Bensterriblescripts/Lib-Handlers/osapi"
)

type HWNDManager struct{}

var activeWindows []osapi.Window

func (h *HWNDManager) GetAllActiveWindows() []osapi.Window {
	activeWindows = osapi.GetAllActiveWindows()
	return activeWindows
}
func (h *HWNDManager) SetBorderlessFullscreen(handle int) {
	osapi.SetBorderlessWindow(uintptr(handle))
}
func (h *HWNDManager) SetWindowed(handle int) {
	osapi.SetWindowWindowed(uintptr(handle))
}
