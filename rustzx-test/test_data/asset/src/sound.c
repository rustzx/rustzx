#include <sound.h>
#include <psg.h>

void main() {
	psg_init();
	psg_channels(chanAll, chanNone); // set all channels to tone generation
    psg_tone(0, psgT(130.8)); // produce a C tone on the first channel
    psg_tone(1, psgT(164.8)); // produce a E tone on the second channel
    psg_tone(2, psgT(195.9)); // produce a G tone on the third channel
    psg_envelope(envUH, psgT(16), chanAll); // set a raising volume envelope on all channels
    bit_play("EmDCDCD"); // play some random tune on beeper
    psg_envelope(envD, psgT(16), chanAll); // set a fading volume envelope on all channels
}
