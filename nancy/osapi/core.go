package osapi

import (
	"os"
	"os/exec"
	"path/filepath"
	"syscall"

	. "github.com/Bensterriblescripts/Lib-Handlers/logging"
)

func Run(command string) (string, bool) {
	cmd := exec.Command("powershell.exe", "-NoProfile", "-NonInteractive", "-Command", command)
	cmd.SysProcAttr = &syscall.SysProcAttr{HideWindow: true} // No powershell window

	out, err := cmd.CombinedOutput()
	if err != nil {
		return string(out), false
	}
	return string(out), true
}
func EnsurePath(path string) bool {
	if ErrExists(os.MkdirAll(filepath.Dir(path), 0755)) {
		ErrorLog("Failed to create directory: " + path)
		return false
	}
	return true
}
func GetFileSize(path string) int64 {
	if info, err := os.Stat(path); err != nil {
		ErrorLog("Failed to get file size")
		return 0
	} else {
		if info.IsDir() {
			ErrorLog("Path is a directory, not a file " + path)
			return 0
		}

		return info.Size()
	}
}
func AddToLocalSoftware() bool {
	if currentExe, err := os.Executable(); err == nil {
		if currentExe == "C:\\Local\\Software\\"+AppName+".exe" {
			TraceLog("The executable is already running from this path...")
			return true
		}

		if _, err := os.Stat("C:\\Local\\Software\\" + AppName + ".exe"); err == nil { // Check if it's already there
			if GetFileSize(currentExe) == GetFileSize("C:\\Local\\Software\\"+AppName+".exe") { // They must be different sizes to bother copying
				TraceLog("No changes were made to the executable, leaving it as is...")
				return true
			}
		}

		if !EnsurePath("C:\\Local\\Software\\") {
			ErrorLog("Failed to ensure the path exists: C:/Local/Software/")
			return false
		}

		out, success := Run(`Copy-Item -Path ` + currentExe + ` -Destination "C:\Local\Software\` + AppName + `.exe" -Force`)
		if !success {
			ErrorLog("Failed to copy self to software: " + out)
			return false
		}
		TraceLog("Copied the current executable into C:/Local/Software/" + AppName + ".exe")
		return true

	} else {
		ErrorLog("Failed to get executable path: " + err.Error())
		return false
	}
}
