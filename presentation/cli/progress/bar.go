package progress

import (
	"fmt"
	"os"
	"strings"
	"time"
)

type ProgressBar struct {
	width      int
	current    int
	total      int
	startTime  time.Time
	lastUpdate time.Time
	message    string
}

func NewProgressBar(total int) *ProgressBar {
	return &ProgressBar{
		width:     50,
		total:     total,
		startTime: time.Now(),
	}
}

func (p *ProgressBar) Update(current int, message string) {
	p.current = current
	p.message = message
	p.lastUpdate = time.Now()
	p.render()
}

func (p *ProgressBar) Finish() {
	p.current = p.total
	p.message = "Complete"
	p.render()
	fmt.Println()
}

func (p *ProgressBar) render() {
	if p.total == 0 {
		return
	}

	percentage := float64(p.current) / float64(p.total)
	if percentage > 1.0 {
		percentage = 1.0
	}

	filled := int(percentage * float64(p.width))
	bar := strings.Repeat("█", filled) + strings.Repeat("░", p.width-filled)

	elapsed := time.Since(p.startTime)
	var eta string
	if p.current > 0 && p.current < p.total {
		avgTimePerItem := elapsed / time.Duration(p.current)
		remaining := time.Duration(p.total-p.current) * avgTimePerItem
		eta = fmt.Sprintf(" ETA: %v", remaining.Round(time.Second))
	}

	status := fmt.Sprintf("\r[%s] %d/%d (%.1f%%) %s%s",
		bar, p.current, p.total, percentage*100, p.message, eta)

	fmt.Print(status)

	if isatty() {
		fmt.Print("\033[K")
	}
}

func isatty() bool {
	fi, err := os.Stdout.Stat()
	if err != nil {
		return false
	}
	return (fi.Mode() & os.ModeCharDevice) != 0
}
