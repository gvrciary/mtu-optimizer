package entities

type ProgressCallback func(current, total int, message string)

type OptimizationProgress struct {
	CurrentMTU  int
	TotalMTUs   int
	CurrentPing int
	TotalPings  int
	Message     string
	IsComplete  bool
}
