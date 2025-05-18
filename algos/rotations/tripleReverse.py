@Rotation("Triple Reversal", RotationMode.INDEXED)
def tripleReversal(array, a, m, b):
    reverse(array, a, m)
    reverse(array, m, b)
    reverse(array, a, b)
