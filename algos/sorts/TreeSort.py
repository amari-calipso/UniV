class TreeSort:
    def traverse(this, array, tmp, lower, upper, r):
        if lower[r] != 0:
            this.traverse(array, tmp, lower, upper, lower[r].readInt())
        tmp[this.idx].write(array[r])
        this.idx += 1
        if upper[r] != 0:
            this.traverse(array, tmp, lower, upper, upper[r].readInt())

    def sort(this, array):
        this.lower = sortingVisualizer.createValueArray(len(array))
        this.upper = sortingVisualizer.createValueArray(len(array))
        this.tmp = sortingVisualizer.createValueArray(len(array))
        sortingVisualizer.setNonOrigAux([this.lower, this.upper])
        i = 1
        while i < len(array):
            c = 0
            while True:
                next: list
                if array[i] < array[c]:
                    next = this.lower
                else:
                    next = this.upper
                if next[c] == 0:
                    next[c].write(i)
                    break
                else:
                    c = next[c].readInt()
            i += 1
        this.idx = 0
        this.traverse(array, this.tmp, this.lower, this.upper, 0)
        arrayCopy(this.tmp, 0, array, 0, len(array))


@Sort(
    "Tree Sorts", 
    "Tree Sort", 
    "Tree Sort",
    {
        "Linear": [
            "Reversed", "Reversed Sawtooth", "No Shuffle", "Sawtooth", 
            "Few Random", "Final Merge Pass", "Real Final Merge", "Noisy",
            "Scrambled Tail", "Scrambled Head", "Sorted", "Quicksort Adversary",
            "Grailsort Adversary"
        ],
        "Quadratic": [
            "Reversed", "Reversed Sawtooth", "No Shuffle", "Sawtooth", 
            "Few Random", "Final Merge Pass", "Real Final Merge", "Noisy",
            "Scrambled Tail", "Scrambled Head", "Sorted", "Quicksort Adversary",
            "Grailsort Adversary"
        ],
        "Quintic":   [
            "Random", "Reversed", "Reversed Sawtooth", "No Shuffle", "Sawtooth", 
            "Few Random", "Final Merge Pass", "Real Final Merge", "Noisy",
            "Scrambled Tail", "Scrambled Head", "Sorted", "Quicksort Adversary",
            "Grailsort Adversary"
        ],
        "Sine Wave": [
            "Reversed", "Reversed Sawtooth", "No Shuffle", "Sawtooth", 
            "Few Random", "Final Merge Pass", "Real Final Merge", "Noisy",
            "Scrambled Tail", "Scrambled Head", "Sorted", "Quicksort Adversary",
            "Grailsort Adversary"
        ],
        "Perlin Noise": [
            "Reversed", "Reversed Sawtooth", "No Shuffle", "Sawtooth", 
            "Few Random", "Final Merge Pass", "Real Final Merge", "Noisy",
            "Scrambled Tail", "Scrambled Head", "Sorted", "Quicksort Adversary",
            "Grailsort Adversary"
        ],
    }
)
def treeSortRun(array):
    TreeSort().sort(array)
