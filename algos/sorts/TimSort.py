class TimSort:
    MIN_MERGE = 32
    MIN_GALLOP = 7
    INITIAL_TMP_STORAGE_LENGTH = 256

    def __init__(this, a, length):
        this.a = a
        this.len = length
        stackLen = 5 if (this.len < 120)else (
            10 if (this.len < 1542)else (19 if (this.len < 119151)else 40))
        this.runBase = sortingVisualizer.createValueArray(stackLen)
        this.runLen = sortingVisualizer.createValueArray(stackLen)
        sortingVisualizer.setNonOrigAux([this.runBase, this.runLen])
        this.tmp = sortingVisualizer.createValueArray((this.len >> 1)if (
            this.len < 2*this.INITIAL_TMP_STORAGE_LENGTH)else this.INITIAL_TMP_STORAGE_LENGTH)
        this.minGallop = this.MIN_GALLOP
        this.stackSize = 0

    def sort(this, a, lo, hi):
        nRemaining = hi-lo
        if nRemaining < this.MIN_MERGE:
            initRunLen: int
            initRunLen = this.countRunAndMakeAscending(a, lo, hi)
            this.binarySort(a, lo, hi, lo+initRunLen)
            return
        minRun = this.minRunLength(nRemaining)
        while True:
            runLen: int
            runLen = this.countRunAndMakeAscending(a, lo, hi)
            if runLen < minRun:
                force = nRemaining if (nRemaining <= minRun)else minRun
                this.binarySort(a, lo, lo+force, lo+runLen)
                runLen = force
            this.pushRun(lo, runLen)
            this.mergeCollapse()
            lo += runLen
            nRemaining -= runLen
            if not (nRemaining != 0):
                break
        this.mergeForceCollapse()

    def binarySort(this, a, lo, hi, start):
        binaryInsertionSort(a, min(lo, start), hi)

    def countRunAndMakeAscending(this, a, lo, hi):
        runHi = lo+1
        if runHi == hi:
            return 1
        if a[runHi] < a[lo]:
            runHi += 1
            while runHi < hi and a[runHi] < a[runHi-1]:
                runHi += 1
            this.reverseRange(a, lo, runHi)
        else:
            runHi += 1
            while runHi < hi and a[runHi] >= a[runHi-1]:
                runHi += 1
        return runHi-lo

    def reverseRange(this, a, lo, hi):
        reverse(a, lo, hi)

    def minRunLength(this, n):
        r = 0
        while n >= this.MIN_MERGE:
            r |= (n & 1)
            n >>= 1
        return n+r

    def pushRun(this, runBase, runLen):
        this.runBase[this.stackSize].write(runBase)
        this.runLen[this.stackSize].write(runLen)
        this.stackSize += 1

    def mergeCollapse(this):
        while this.stackSize > 1:
            n = this.stackSize-2
            if (n >= 1 and this.runLen[n-1] <= this.runLen[n]+this.runLen[n+1]) or (n >= 2 and this.runLen[n-2] <= this.runLen[n]+this.runLen[n-1]):
                if this.runLen[n-1] < this.runLen[n+1]:
                    n -= 1
            elif this.runLen[n] > this.runLen[n+1]:
                break
            this.mergeAt(n)

    def mergeForceCollapse(this):
        while this.stackSize > 1:
            n = this.stackSize-2
            if n > 0 and this.runLen[n-1] < this.runLen[n+1]:
                n -= 1
            this.mergeAt(n)

    def mergeAt(this, i):
        base1 = this.runBase[i].readInt()
        len1 = this.runLen[i].readInt()
        base2 = this.runBase[i+1].readInt()
        len2 = this.runLen[i+1].readInt()
        this.runLen[i].write(len1+len2)
        if i == this.stackSize-3:
            this.runBase[i+1].write(this.runBase[i+2])
            this.runLen[i+1].write(this.runLen[i+2])
        this.stackSize -= 1
        k: int
        k = this.gallopRight(this.a[base2], this.a, base1, len1, 0)
        base1 += k
        len1 -= k
        if len1 == 0:
            return
        len2 = this.gallopLeft(
            this.a[base1+len1-1], this.a, base2, len2, len2-1)
        if len2 == 0:
            return
        if len1 <= len2:
            this.mergeLo(base1, len1, base2, len2)
        else:
            this.mergeHi(base1, len1, base2, len2)

    def gallopLeft(this, key, a, base, len, hint):
        lastOfs = 0
        ofs = 1
        if key > a[base+hint]:
            maxOfs = len-hint
            while ofs < maxOfs and key > a[base+hint+ofs]:
                lastOfs = ofs
                ofs = (ofs*2)+1
                if ofs <= 0:
                    ofs = maxOfs
            if ofs > maxOfs:
                ofs = maxOfs
            lastOfs += hint
            ofs += hint
        else:
            maxOfs = hint+1
            while ofs < maxOfs and key <= a[base+hint-ofs]:
                lastOfs = ofs
                ofs = (ofs*2)+1
                if ofs <= 0:
                    ofs = maxOfs
            if ofs > maxOfs:
                ofs = maxOfs
            tmp = lastOfs
            lastOfs = hint-ofs
            ofs = hint-tmp
        lastOfs += 1
        while lastOfs < ofs:
            m = lastOfs+((ofs-lastOfs) >> 1)
            if key > a[base+m]:
                lastOfs = m+1
            else:
                ofs = m
        return ofs

    def gallopRight(this, key, a, base, len, hint):
        ofs = 1
        lastOfs = 0
        if key < a[base+hint]:
            maxOfs = hint+1
            while ofs < maxOfs and key < a[base+hint-ofs]:
                lastOfs = ofs
                ofs = (ofs*2)+1
                if ofs <= 0:
                    ofs = maxOfs
            if ofs > maxOfs:
                ofs = maxOfs
            tmp = lastOfs
            lastOfs = hint-ofs
            ofs = hint-tmp
        else:
            maxOfs = len-hint
            while ofs < maxOfs and key >= a[base+hint+ofs]:
                lastOfs = ofs
                ofs = (ofs*2)+1
                if ofs <= 0:
                    ofs = maxOfs
            if ofs > maxOfs:
                ofs = maxOfs
            lastOfs += hint
            ofs += hint
        lastOfs += 1
        while lastOfs < ofs:
            m = lastOfs+((ofs-lastOfs) >> 1)
            if key < a[base+m]:
                ofs = m
            else:
                lastOfs = m+1
        return ofs

    def mergeLo(this, base1, len1, base2, len2):
        a = this.a
        tmp = this.ensureCapacity(len1)
        arrayCopy(a, base1, tmp, 0, len1)
        cursor1 = 0
        cursor2 = base2
        dest = base1
        a[dest].write(a[cursor2])
        dest += 1
        cursor2 += 1
        len2 -= 1
        if len2 == 0:
            arrayCopy(tmp, cursor1, a, dest, len1)
            return
        if len1 == 1:
            arrayCopy(a, cursor2, a, dest, len2)
            a[dest+len2].write(tmp[cursor1])
            return
        minGallop = this.minGallop
        breakOuter = False
        while True:
            count1 = 0
            count2 = 0
            while True:
                if a[cursor2] < tmp[cursor1]:
                    a[dest].write(a[cursor2])
                    cursor2 += 1
                    dest += 1
                    count2 += 1
                    count1 = 0
                    len2 -= 1
                    if len2 == 0:
                        breakOuter = True
                        break
                else:
                    a[dest].write(tmp[cursor1])
                    dest += 1
                    cursor1 += 1
                    count1 += 1
                    count2 = 0
                    len1 -= 1
                    if len1 == 1:
                        breakOuter = True
                        break
                if not ((count1 | count2) < minGallop):
                    break
            if breakOuter:
                break
            while True:
                count1 = this.gallopRight(a[cursor2], tmp, cursor1, len1, 0)
                if count1 != 0:
                    arrayCopy(tmp, cursor1, a, dest, count1)
                    dest += count1
                    cursor1 += count1
                    len1 -= count1
                    if len1 <= 1:
                        breakOuter = True
                        break
                a[dest].write(a[cursor2])
                dest += 1
                cursor2 += 1
                len2 -= 1
                if len2 == 0:
                    breakOuter = True
                    break
                count2 = this.gallopLeft(tmp[cursor1], a, cursor2, len2, 0)
                if count2 != 0:
                    arrayCopy(a, cursor2, a, dest, count2)
                    dest += count2
                    cursor2 += count2
                    len2 -= count2
                    if len2 == 0:
                        breakOuter = True
                        break
                a[dest].write(tmp[cursor1])
                dest += 1
                cursor1 += 1
                len1 -= 1
                if len1 == 1:
                    breakOuter = True
                    break
                minGallop -= 1
                if not (count1 >= this.MIN_GALLOP | count2 >= this.MIN_GALLOP):
                    break
            if breakOuter:
                break
            if minGallop < 0:
                minGallop = 0
            minGallop += 2
        this.minGallop = 1 if (minGallop < 1)else minGallop
        if len1 == 1:
            arrayCopy(a, cursor2, a, dest, len2)
            a[dest+len2].write(tmp[cursor1])
        elif (len1 == 0):
            print("Comparison method violates its general contract!\n")
            return
        else:
            arrayCopy(tmp, cursor1, a, dest, len1)

    def mergeHi(this, base1, len1, base2, len2):
        a = this.a
        tmp = this.ensureCapacity(len2)
        arrayCopy(a, base2, tmp, 0, len2)
        cursor1 = base1+len1-1
        cursor2 = len2-1
        dest = base2+len2-1
        a[dest].write(a[cursor1])
        dest -= 1
        cursor1 -= 1
        len1 -= 1
        if len1 == 0:
            reverseArrayCopy(tmp, 0, a, dest-(len2-1), len2)
            return
        if len2 == 1:
            dest -= len1
            cursor1 -= len1
            reverseArrayCopy(a, cursor1+1, a, dest+1, len1)
            a[dest].write(tmp[cursor2])
            return
        minGallop = this.minGallop
        breakOuter = False
        while True:
            count1 = 0
            count2 = 0
            while True:
                if tmp[cursor2] < a[cursor1]:
                    a[dest].write(a[cursor1])
                    dest -= 1
                    cursor1 -= 1
                    count1 += 1
                    count2 = 0
                    len1 -= 1
                    if len1 == 0:
                        breakOuter = True
                        break
                else:
                    a[dest].write(tmp[cursor2])
                    dest -= 1
                    cursor2 -= 1
                    count2 += 1
                    count1 = 0
                    len2 -= 1
                    if len2 == 1:
                        breakOuter = True
                        break
                if not ((count1 | count2) < minGallop):
                    break
            if breakOuter:
                break
            while True:
                count1 = len1 - \
                    this.gallopRight(tmp[cursor2], a, base1, len1, len1-1)
                if count1 != 0:
                    dest -= count1
                    cursor1 -= count1
                    len1 -= count1
                    reverseArrayCopy(a, cursor1+1, a, dest+1, count1)
                    if len1 == 0:
                        breakOuter = True
                        break
                a[dest].write(tmp[cursor2])
                dest -= 1
                cursor2 -= 1
                len2 -= 1
                if len2 == 1:
                    breakOuter = True
                    break
                count2 = len2-this.gallopLeft(a[cursor1], tmp, 0, len2, len2-1)
                if count2 != 0:
                    dest -= count2
                    cursor2 -= count2
                    len2 -= count2
                    reverseArrayCopy(tmp, cursor2+1, a, dest+1, count2)
                    if len2 <= 1:
                        breakOuter = True
                        break
                a[dest].write(a[cursor1])
                dest -= 1
                cursor1 -= 1
                len1 -= 1
                if len1 == 0:
                    breakOuter = True
                    break
                minGallop -= 1
                if not (count1 >= this.MIN_GALLOP | count2 >= this.MIN_GALLOP):
                    break
            if breakOuter:
                break
            if minGallop < 0:
                minGallop = 0
            minGallop += 2
        this.minGallop = 1 if (minGallop < 1)else minGallop
        if len2 == 1:
            dest -= len1
            cursor1 -= len1
            reverseArrayCopy(a, cursor1+1, a, dest+1, len1)
            a[dest].write(tmp[cursor2])
        elif (len2 == 0):
            print("Comparison method violates its general contract!\n")
            return
        else:
            reverseArrayCopy(tmp, 0, a, dest-(len2-1), len2)

    def ensureCapacity(this, minCapacity):
        if len(this.tmp) < minCapacity:
            newSize = minCapacity
            newSize |= newSize >> 1
            newSize |= newSize >> 2
            newSize |= newSize >> 4
            newSize |= newSize >> 8
            newSize |= newSize >> 16
            newSize += 1
            if newSize < 0:
                newSize = minCapacity
            else:
                newSize = min(newSize, this.len >> 1)
            newArray = sortingVisualizer.createValueArray(newSize)
            this.tmp = newArray
        return this.tmp


@Sort("Merge Sorts", "Tim Sort", "Tim Sort")
def timSortRun(array):
    TimSort(array, len(array)).sort(array, 0, len(array))
