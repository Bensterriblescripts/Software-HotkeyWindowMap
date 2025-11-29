package yourpkg

import "testing"

type Item struct {
	Prop int
}

func makeSlice(n int) []Item {
	s := make([]Item, n)
	// make only one non-zero, or random positions if you like
	s[n-1].Prop = 1
	return s
}

func BenchmarkScanSlice(b *testing.B) {
	s := makeSlice(100) // change sizes: 10, 50, 100, 500...
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		for i, v := range s {
			if v.Prop != 0 {
				_ = i
				break
			}
		}
	}
}

func BenchmarkMapLookup(b *testing.B) {
	s := makeSlice(100)
	m := make(map[int]int)
	for i, v := range s {
		if v.Prop != 0 {
			m[0] = i // pretend key = 0
		}
	}

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		_ = m[0]
	}
}
