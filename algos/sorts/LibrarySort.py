class LibrarySort:
    R = 4

    def getMinLevel(this, n):
        while n >= 32:
            n = (n-1)//this.R+1
        return n

    def rebalance(this, array, temp, m, b):
        i = 0
        while i < m:
            this.cnts[i+1] += this.cnts[i]+1
            i += 1
        i = m
        j = 0
        while i < b:
            temp[this.cnts[this.locs[j].readInt()].readInt()].write(array[i])
            this.cnts[this.locs[j].getInt()] += 1
            i += 1
            j += 1
        i = 0
        while i < m:
            temp[this.cnts[i].readInt()].write(array[i])
            this.cnts[i] += 1
            i += 1
        arrayCopy(temp, 0, array, 0, b)
        binaryInsertionSort(array, 0, this.cnts[0].readInt()-1)
        i = 0
        while i < m-1:
            binaryInsertionSort(
                array, this.cnts[i].readInt(), this.cnts[i+1].readInt()-1)
            i += 1
        binaryInsertionSort(
            array, this.cnts[m-1].readInt(), this.cnts[m].readInt())
        i = 0
        while i < m+2:
            this.cnts[i].write(0)
            i += 1

    def sort(this, array, length):
        if length < 32:
            binaryInsertionSort(array, 0, length)
            return
        j = this.getMinLevel(length)
        binaryInsertionSort(array, 0, j)
        maxLevel = j
        while maxLevel*this.R < length:
            maxLevel *= this.R
        this.temp = sortingVisualizer.createValueArray(length)
        this.cnts = sortingVisualizer.createValueArray(maxLevel+2)
        this.locs = sortingVisualizer.createValueArray(length-maxLevel)
        sortingVisualizer.setNonOrigAux([this.cnts, this.locs])
        i = j
        k = 0
        while i < length:
            if this.R*j == i:
                this.rebalance(array, this.temp, j, i)
                j = i
                k = 0
            loc: int
            loc = lrBinarySearch(array, 0, j, array[i], False)
            this.cnts[loc+1] += 1
            this.locs[k].write(loc)
            k += 1
            i += 1
        this.rebalance(array, this.temp, j, length)


@Sort(
    "Insertion Sorts", 
    "Library Sort", 
    "Library Sort",
    { # killers
        "Linear":    ["Reversed", "Reversed Sawtooth", "Partitioned"], 
        "Quadratic": ["Reversed", "Reversed Sawtooth", "Partitioned"], 
        "Quintic":   ["Reversed", "Reversed Sawtooth", "Partitioned", "Final Merge Pass", "Real Final Merge"], 
        "Sine Wave": [
            "Reversed", "Reversed Sawtooth", "No shuffle", "Sawtooth", 
            "Partitioned", "Final Merge Pass", "Noisy", "Real Final Merge",
            "Scrambled Tail", "Scrambled Head"
        ],
        "Perlin Noise": ["Partitioned", "Real Final Merge"]
    }
)
def librarySortRun(array):
    LibrarySort().sort(array, len(array))
