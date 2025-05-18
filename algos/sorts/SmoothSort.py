class SmoothSort:
    LP = [1, 1, 3, 5, 9, 15, 25, 41, 67, 109, 177, 287, 465,
          753, 1219, 1973, 3193, 5167, 8361, 13529, 21891]

    def sift(this, array, pshift, head):
        val = array[head].copy()
        while pshift > 1:
            rt = head-1
            lf = rt-this.LP[pshift-2]
            if val >= array[lf] and val >= array[rt]:
                break
            if array[lf] >= array[rt]:
                array[head].write(array[lf])
                head = lf
                pshift -= 1
            else:
                array[head].write(array[rt])
                head = rt
                pshift -= 2
        array[head].write(val)

    def trinkle(this, array, p, pshift, head, isTrusty):
        val = array[head].copy()
        while p != 1:
            stepson = head-this.LP[pshift]
            if array[stepson] <= val:
                break
            if (not isTrusty) and pshift > 1:
                rt = head-1
                lf = rt-this.LP[pshift-2]
                if array[rt] >= array[stepson] or array[lf] >= array[stepson]:
                    break
            array[head].write(array[stepson])
            head = stepson
            trail = javaNumberOfTrailingZeros(p & 4294967294) # 0xfffffffe
            p >>= trail
            pshift += trail
            isTrusty = False
        if not isTrusty:
            array[head].write(val)
            this.sift(array, pshift, head)

    def sort(this, array, lo, hi):
        head = lo
        p = 1
        pshift = 1
        while head < hi:
            if (p & 3) == 3:
                this.sift(array, pshift, head)
                p >>= 2
                pshift += 2
            else:
                if this.LP[pshift-1] >= hi-head:
                    this.trinkle(array, p, pshift, head, False)
                else:
                    this.sift(array, pshift, head)
                if pshift == 1:
                    p <<= 1
                    pshift -= 1
                else:
                    p <<= (pshift-1)
                    pshift = 1
            p |= 1
            head += 1
        this.trinkle(array, p, pshift, head, False)
        while pshift != 1 or p != 1:
            if pshift <= 1:
                trail = javaNumberOfTrailingZeros(p & 4294967294) # 0xfffffffe
                p >>= trail
                pshift += trail
            else:
                p <<= 2
                p ^= 7
                pshift -= 2
                this.trinkle(array, p >> 1, pshift+1, head -
                             this.LP[pshift]-1, True)
                this.trinkle(array, p, pshift, head-1, True)
            head -= 1


@Sort("Tree Sorts", "Smooth Sort", "Smooth Sort",)
def smoothSortRun(array):
    SmoothSort().sort(array, 0, len(array)-1)
