package output

import (
	"encoding/json"
	"fmt"
	"mtu-optimizer/domain/entities"
	"sort"
	"strings"
)

type Config struct {
	Format  string
	Verbose bool
	ShowAll bool
}

func DisplayOptimizationResult(result *entities.OptimizationResult, config *Config) error {
	switch config.Format {
	case "json":
		return displayJSON(result)
	case "csv":
		return displayCSV(result)
	default:
		return displayTable(result, config)
	}
}

func DisplayMTUTestResult(result *entities.MTUTestResult, config *Config) error {
	switch config.Format {
	case "json":
		return displayJSON(result)
	default:
		return displaySingleMTU(result, config)
	}
}

func displayTable(result *entities.OptimizationResult, config *Config) error {
	if len(result.AllResults) == 0 {
		fmt.Println("No results obtained")
		return nil
	}

	fmt.Println("MTU OPTIMIZATION RESULTS")
	fmt.Println(strings.Repeat("=", 50))

	if result.BestMTU != nil {
		jitterMs := float64(result.BestMTU.Jitter.Nanoseconds()) / 1000000.0
		fmt.Printf("OPTIMAL MTU: %d\n", result.BestMTU.MTU)
		fmt.Printf("  Average latency: %v\n", result.BestMTU.AvgLatency)
		fmt.Printf("  Packet loss: %.1f%%\n", result.BestMTU.PacketLossRate*100)
		fmt.Printf("  Jitter: %.3fms\n", jitterMs)
	}

	if result.WorstMTU != nil {
		fmt.Printf("\nWORST MTU: %d\n", result.WorstMTU.MTU)
		fmt.Printf("  Average latency: %v\n", result.WorstMTU.AvgLatency)
		fmt.Printf("  Packet loss: %.1f%%\n", result.WorstMTU.PacketLossRate*100)
	}

	fmt.Printf("\nSTATISTICS\n")
	fmt.Printf("  MTUs tested: %d\n", len(result.AllResults))
	fmt.Printf("  Packets sent: %d\n", result.TotalPackets)
	fmt.Printf("  Total time: %v\n", result.TotalDuration)

	if config.ShowAll {
		fmt.Printf("\nALL RESULTS\n")
		fmt.Println(strings.Repeat("-", 80))
		fmt.Printf("%-6s %-6s %-12s %-12s %-12s %-10s %-10s\n",
			"Status", "MTU", "Min", "Avg", "Max", "Loss %", "Jitter")
		fmt.Println(strings.Repeat("-", 80))

		sorted := make([]entities.MTUTestResult, len(result.AllResults))
		copy(sorted, result.AllResults)
		sort.Slice(sorted, func(i, j int) bool {
			return sorted[i].MTU < sorted[j].MTU
		})

		for _, r := range sorted {
			status := "OK"
			if !r.Success {
				status = "FAIL"
			}

			jitterMs := float64(r.Jitter.Nanoseconds()) / 1000000.0
			fmt.Printf("%-6s %-6d %-12v %-12v %-12v %-9.1f%% %.3fms\n",
				status, r.MTU, r.MinLatency, r.AvgLatency, r.MaxLatency,
				r.PacketLossRate*100, jitterMs)
		}
	}

	return nil
}

func displaySingleMTU(result *entities.MTUTestResult, config *Config) error {
	fmt.Printf("MTU %d RESULTS\n", result.MTU)
	fmt.Println(strings.Repeat("=", 30))

	if !result.Success {
		fmt.Println("TEST FAILED")
		fmt.Printf("Packet loss: %.1f%%\n", result.PacketLossRate*100)
		return nil
	}

	jitterMs := float64(result.Jitter.Nanoseconds()) / 1000000.0
	fmt.Printf("Min latency: %v\n", result.MinLatency)
	fmt.Printf("Avg latency: %v\n", result.AvgLatency)
	fmt.Printf("Max latency: %v\n", result.MaxLatency)
	fmt.Printf("Jitter: %.3fms\n", jitterMs)
	fmt.Printf("Packet loss: %.1f%%\n", result.PacketLossRate*100)

	if config.Verbose {
		fmt.Printf("\nINDIVIDUAL PINGS\n")
		fmt.Println(strings.Repeat("-", 40))
		for i, ping := range result.Pings {
			status := "OK"
			if !ping.Success {
				status = "FAIL"
			}
			fmt.Printf("%d. %s %v (payload: %d bytes)\n",
				i+1, status, ping.Latency, ping.PayloadSize)
		}
	}

	return nil
}

func displayJSON(data interface{}) error {
	jsonData, err := json.MarshalIndent(data, "", "  ")
	if err != nil {
		return err
	}
	fmt.Println(string(jsonData))
	return nil
}

func displayCSV(result *entities.OptimizationResult) error {
	fmt.Println("MTU,MinLatency,AvgLatency,MaxLatency,PacketLoss,Jitter,Success")
	for _, r := range result.AllResults {
		fmt.Printf("%d,%v,%v,%v,%.2f,%v,%t\n",
			r.MTU, r.MinLatency, r.AvgLatency, r.MaxLatency,
			r.PacketLossRate, r.Jitter, r.Success)
	}
	return nil
}
