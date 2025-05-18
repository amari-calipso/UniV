class RadixSort:
    def __init__(this, base):
        if base is None:
            this.base = sortingVisualizer.getUserInput(
                "Insert base: ", "4", parseInt)
        else:
            this.base = base
        this.arrayLen = None
        this.offs = 0

    def getHighestPower(this, array, a, b):
        return findHighestPower(array, a, b, this.base)

    def transcribe(this, array, a, aux, empty):
        i = a
        for j in range(len(aux)):
            for element in aux[j]:
                array[i].write(element)
                i += 1
            
        if empty:
            for j in range(len(aux)):
                List_clear(aux[j])

    def LSD(this, array, a, b):
        this.arrayLen = b-a
        hPow = this.getHighestPower(array, a, b)
        aux = []
        for _ in range(this.base):
            List_invisiblePush(aux, Array(0))
        for p in range(hPow+1):
            i = a
            while i < b:
                dig = array[i].readDigit(p, this.base)
                List_push(aux[dig], array[i])
                i += 1
            this.transcribe(array, a, aux, True)

    def MSD(this, array, a, b, p):
        if p is None:
            p = this.getHighestPower(array, a, b)
        if a >= b or p < -1:
            return
        aux = []
        for _ in range(this.base):
            List_invisiblePush(aux, Array(0))
        i = a
        while i < b:
            dig = array[i].readDigit(p, this.base)
            List_push(aux[dig], array[i])
            i += 1
        this.transcribe(array, a, aux, False)
        this.offs += b-a
        sum_ = 0
        for i in range(len(aux)):
            this.MSD(array, a+sum_, a+sum_+len(aux[i]), p-1)
            sum_ += len(aux[i])
            this.offs -= len(aux[i])
            List_clear(aux[i])

    def inPlaceLSD(this, array, length):
        pos = 0
        vregs = StandaloneArray(this.base-1)
        maxpower = this.getHighestPower(array, 0, length)
        p = 0
        while p <= maxpower:
            for i in range(len(vregs)):
                vregs[i] = length-1
            pos = 0
            for i in range(length):
                dig = array[pos].readDigit(p, this.base)
                if dig == 0:
                    pos += 1
                    sortingVisualizer.markArray(0, pos)
                else:
                    for j in range(len(vregs)):
                        sortingVisualizer.markArray(j+1, vregs[j])
                    i = pos
                    while i < vregs[dig-1]:
                        swap(array, i, i+1)
                        i += 1
                    j = dig-1
                    while j > 0:
                        vregs[j-1] = vregs[j-1]-1
                        j -= 1
            p += 1


@Sort("Distribution Sorts", "Least Significant Digit Radix Sort", "LSD Radix Sort")
def LSDRadixSortRun(array):
    RadixSort(None).LSD(array, 0, len(array))


@Sort("Distribution Sorts", "Most Significant Digit Radix Sort", "MSD Radix Sort")
def MSDRadixSortRun(array):
    RadixSort(None).MSD(array, 0, len(array), None)


@Sort("Distribution Sorts", "In-Place LSD Radix Sort", "In-Place LSD Radix Sort",)
def inPlaceLSDRadixSortRun(array):
    RadixSort(None).inPlaceLSD(array, len(array))
