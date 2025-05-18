class NewShuffleMergeSort:
    def __init__(this, rot):
        if rot is None:
            this.__rotate = UniV_getUserRotation("Select rotation algorithm (default: Gries-Mills)").lengths
        else:
            this.__rotate = sortingVisualizer.getRotationByName(rot).lengths

    def rotate(this, array, m, a, b):
        rotate = this.__rotate
        rotate(array, m-a, a, b)

    def shuffleEasy(this, array, start, size):
        i = 1
        while i < size:
            val = array[start+i-1].read()
            j = (i*2) % size
            while j != i:
                nval = array[start+j-1].read()
                array[start+j-1].write(val)
                val = nval
                j = (j*2) % size
            array[start+i-1].write(val)
            i *= 3

    def shuffle(this, array, start, end):
        while end-start > 1:
            n = (end-start)//2
            l = 1
            while l*3-1 <= 2*n:
                l *= 3
            m = (l-1)//2
            this.rotate(array, start+n, n-m, m)
            this.shuffleEasy(array, start, l)
            start += l-1

    def rotateShuffledEqual(this, array, a, b, size):
        i = 0
        while i < size:
            array[a+i].swap(array[b+i])
            i += 2

    def rotateShuffled(this, array, mid, a, b):
        while a > 0 and b > 0:
            if a > b:
                this.rotateShuffledEqual(array, mid-b, mid, b)
                mid -= b
                a -= b
            else:
                this.rotateShuffledEqual(array, mid-a, mid, a)
                mid += a
                b -= a

    def rotateShuffledOuter(this, array, mid, a, b):
        if a > b:
            this.rotateShuffledEqual(array, mid-b, mid+1, b)
            mid -= b
            a -= b
        else:
            this.rotateShuffledEqual(array, mid-a, mid+1, a)
            mid += a+1
            b -= a
        this.rotateShuffled(array, mid, a, b)

    def unshuffleEasy(this, array, start, size):
        i = 1
        while i < size:
            prev = i
            val = array[start+i-1].read()
            j = (i*2) % size
            while j != i:
                array[start+prev-1].write(array[start+j-1])
                prev = j
                j = (j*2) % size
            array[start+prev-1].write(val)
            i *= 3

    def unshuffle(this, array, start, end):
        while end-start > 1:
            n = (end-start)//2
            l = 1
            while l*3-1 <= 2*n:
                l *= 3
            m = (l-1)//2
            this.rotateShuffledOuter(array, start+2*m, 2*m, 2*n-2*m)
            this.unshuffleEasy(array, start, l)
            start += l-1

    def mergeUp(this, array, start, end, type_):
        i = start
        j = i+1
        while j < end:
            cmp = compareValues(array[i], array[j])
            if cmp < 0 or (not type_) and cmp == 0:
                i += 1
                if i == j:
                    j += 1
                    type_ = not type_
            elif end-j == 1:
                this.rotate(array, j, j-i, 1)
                break
            else:
                r = 0
                if type_:
                    while j+2*r < end and compareValues(array[j+2*r], array[i]) != 1:
                        r += 1
                else:
                    while j+2*r < end and array[j+2*r] < array[i]:
                        r += 1
                j -= 1
                this.unshuffle(array, j, j+2*r)
                this.rotate(array, j, j-i, r)
                i += r+1
                j += 2*r+1

    def merge(this, array, start, mid, end):
        if mid-start <= end-mid:
            this.shuffle(array, start, end)
            this.mergeUp(array, start, end, True)
        else:
            this.shuffle(array, start+1, end)
            this.mergeUp(array, start, end, False)

    def sort(this, array, a, b):
        i = a
        while i < b-1:
            compSwap(array, i, i+1)
            i += 2
        r = 2
        while r < b-a:
            twoR = r*2
            i = a
            while i < b-twoR:
                this.merge(array, i, i+r, i+twoR)
                i += twoR
            if i+r < b:
                this.merge(array, i, i+r, b)
            r = twoR


@Sort("Merge Sorts", "New Shuffle Merge Sort", "New Shuffle Merge")
def newShuffleMergeSortRun(array):
    NewShuffleMergeSort(None).sort(array, 0, len(array))
