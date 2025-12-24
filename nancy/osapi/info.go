package osapi

import "unsafe"

func GetScreenSize() (width, height int32) {
	w := GetSystemMetrics(SM_CXSCREEN)
	h := GetSystemMetrics(SM_CYSCREEN)
	return w, h
}
func GetSystemMetrics(index int32) int32 {
	r, _, _ := procGetSystemMetrics.Call(uintptr(index))
	return int32(r)
}
func GetMessage(msg *MSG, hwnd uintptr, msgFilterMin uint32, msgFilterMax uint32) int32 {
	r, _, _ := procGetMessageW.Call(
		uintptr(unsafe.Pointer(msg)),
		hwnd,
		uintptr(msgFilterMin),
		uintptr(msgFilterMax),
	)
	return int32(r)
}
