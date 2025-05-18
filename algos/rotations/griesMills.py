@Rotation("Gries-Mills", RotationMode.LENGTHS)
def griesMills(array, start, leftLen, rightLen):
    while leftLen > 0 and rightLen > 0:
        if leftLen <= rightLen:
            blockSwap(array, start, start+leftLen, leftLen)
            start += leftLen
            rightLen -= leftLen
        else:
            blockSwap(array, start+leftLen-rightLen, start+leftLen, rightLen)
            leftLen -= rightLen
