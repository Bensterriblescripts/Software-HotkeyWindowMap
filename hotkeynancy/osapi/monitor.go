package osapi

import (
	"fmt"
	"os"

	. "github.com/Bensterriblescripts/Lib-Handlers/logging"
	proc "github.com/shirou/gopsutil/v4/process"
)

func PrintProcUsage() {
	p, err := proc.NewProcess(int32(os.Getpid()))
	if err != nil {
		ErrorLog(fmt.Sprintf("Failed to get process: %v", err))
	}
	cpuPercent, err := p.CPUPercent()
	if err != nil {
		ErrorLog(fmt.Sprintf("Failed to get CPU percent: %v", err))
	}

	mem, _ := p.MemoryInfo()
	TraceLog(fmt.Sprintf("CPU: %.2f%%, Resident Memory: %d mb, Virtual Memory: %d mb", cpuPercent, mem.RSS/1024/1024, mem.VMS/1024/1024))
}
