use anyhow::Context as _;
use anyhow::bail;
use lewton::inside_ogg::OggStreamReader;
use std::io;

#[derive(Debug)]
pub struct AudioSource {
    pub samples: Vec<Vec<f32>>,
    pub sample_rate: u32,
    pub channels: usize,
    pub len: usize,
}

impl AudioSource {
    pub fn from_ogg<T>(
        stream: &mut OggStreamReader<T>,
        output_channels: usize,
    ) -> anyhow::Result<Self>
    where
        T: io::Read + io::Seek,
    {
        let in_channels = stream.ident_hdr.audio_channels as usize;

        let mut samples = vec![Vec::new(); output_channels];
        while let Some(pck) = stream.read_dec_packet_itl()? {
            for frame in pck.chunks_exact(in_channels) {
                let mut frame_f32 = frame.iter().map(|v| *v as f32 / 32768.0);
                if in_channels == output_channels {
                    for channel in &mut samples {
                        channel.push(frame_f32.next().unwrap_or_default());
                    }
                } else if in_channels == 1 {
                    let value = frame_f32.next().unwrap_or_default();
                    for channel in &mut samples {
                        channel.push(value);
                    }
                } else if output_channels == 1 {
                    samples[0].push(frame_f32.sum::<f32>() / in_channels as f32);
                } else {
                    bail!(
                        "Unable to convert channels from {} to {}.",
                        in_channels,
                        output_channels
                    );
                }
            }
        }

        let len = stream
            .get_last_absgp()
            .context("Failed to get data length")? as usize;

        Ok(Self {
            samples,
            sample_rate: stream.ident_hdr.audio_sample_rate,
            channels: output_channels,
            len,
        })
    }
}
