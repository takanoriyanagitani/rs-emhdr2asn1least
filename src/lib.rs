use std::borrow::Cow;
use std::io;

use io::Read;

use io::BufWriter;
use io::Write;

use mailparse::MailHeader;

pub struct LeastMailInfo<'a> {
    pub from: Cow<'a, str>,
    pub to1st: Cow<'a, str>,
    pub subject: Cow<'a, str>,
    /// Unixtime in seconds
    pub date: i64,
}

impl<'a> LeastMailInfo<'a> {
    pub fn to_der<W>(&self, wtr: &mut W) -> Result<(), io::Error>
    where
        W: Write,
    {
        let sf: &str = &self.from;
        let st: &str = &self.to1st;
        let ss: &str = &self.subject;

        let bf: &[u8] = sf.as_bytes();
        let bt: &[u8] = st.as_bytes();
        let bs: &[u8] = ss.as_bytes();

        let chkf: bool = bf.len() < 128;
        let chkt: bool = bt.len() < 128;
        let chks: bool = bs.len() < 128;

        let ok: bool = chkf && chkt && chks;
        let ng: bool = !ok;
        if ng {
            return Err(io::Error::other("non-short header"));
        }

        let bdate: [u8; 8] = self.date.to_le_bytes();

        let seqsz: usize = (2 + bf.len()) + (2 + bt.len()) + (2 + bs.len()) + (2 + bdate.len());
        let i7sz: i8 = seqsz
            .try_into()
            .map_err(|_| io::Error::other("non short header"))?;

        // sequence tag
        wtr.write_all(&[0x30])?;
        // size of the sequence
        wtr.write_all(&[i7sz as u8])?;

        // from
        wtr.write_all(&[0x80])?;
        wtr.write_all(&[bf.len() as u8])?;
        wtr.write_all(bf)?;

        // to
        wtr.write_all(&[0x81])?;
        wtr.write_all(&[bt.len() as u8])?;
        wtr.write_all(bt)?;

        // subject
        wtr.write_all(&[0x82])?;
        wtr.write_all(&[bs.len() as u8])?;
        wtr.write_all(bs)?;

        // unixtime
        wtr.write_all(&[0x83])?;
        wtr.write_all(&[0x08])?;
        wtr.write_all(&bdate)?;

        Ok(())
    }
}

#[derive(Default)]
pub struct LeastMailInfoPartial<'a> {
    pub from: Option<Cow<'a, str>>,
    pub to1st: Option<Cow<'a, str>>,
    pub subject: Option<Cow<'a, str>>,
    pub date: Option<Cow<'a, str>>,
}

impl<'a> LeastMailInfoPartial<'a> {
    pub fn to_least_info(self) -> Option<LeastMailInfo<'a>> {
        let from: Cow<str> = self.from?;
        let to1st: Cow<str> = self.to1st.unwrap_or_default();
        let subject: Cow<str> = self.subject.unwrap_or_default();
        let date: Cow<str> = self.date?;
        let oparsed: Option<i64> = mailparse::dateparse(&date).ok();
        let parsed: i64 = oparsed?;
        Some(LeastMailInfo {
            from,
            to1st,
            subject,
            date: parsed,
        })
    }
}

pub fn headers2partial<'a>(hdrs: &'a [MailHeader<'a>]) -> LeastMailInfoPartial<'a> {
    let mut ret = LeastMailInfoPartial::default();

    for hdr in hdrs {
        let key: &str = &hdr.get_key_ref();

        if "Date" == key {
            let val: &[u8] = hdr.get_value_raw();
            let sval: &str = std::str::from_utf8(val).unwrap_or_default();
            ret.date = Some(Cow::Borrowed(sval));
        }

        if "Subject" == key {
            let val: &[u8] = hdr.get_value_raw();
            let sval: &str = std::str::from_utf8(val).unwrap_or_default();
            ret.subject = Some(Cow::Borrowed(sval));
        }

        if "From" == key {
            let val: &[u8] = hdr.get_value_raw();
            let sval: &str = std::str::from_utf8(val).unwrap_or_default();
            ret.from = Some(Cow::Borrowed(sval));
        }

        if "To" == key {
            if ret.to1st.is_some() {
                continue;
            }
            let val: &[u8] = hdr.get_value_raw();
            let sval: &str = std::str::from_utf8(val).unwrap_or_default();
            ret.to1st = Some(Cow::Borrowed(sval));
        }
    }

    ret
}

pub fn hdr_bytes2hdrs2least2wtr<W>(hdr_bytes: &[u8], wtr: &mut W) -> Result<(), io::Error>
where
    W: FnMut(&LeastMailInfo) -> Result<(), io::Error>,
{
    let pair = mailparse::parse_headers(hdr_bytes).map_err(io::Error::other)?;
    let (hdrs, _) = pair;
    let partial: LeastMailInfoPartial = headers2partial(&hdrs);
    let oinfo: Option<LeastMailInfo> = partial.to_least_info();
    let info: LeastMailInfo = oinfo.ok_or_else(|| io::Error::other("invalid mail header"))?;
    wtr(&info)?;
    Ok(())
}

pub fn hdr_bytes2hdrs2least2iowtr<W>(hdr_bytes: &[u8], mut wtr: W) -> Result<(), io::Error>
where
    W: Write,
{
    hdr_bytes2hdrs2least2wtr(hdr_bytes, &mut |inf: &LeastMailInfo| {
        inf.to_der(&mut wtr)?;
        wtr.flush()
    })
}

pub fn stdin2hdrs2der2stdout(buf: &mut Vec<u8>, lmt: u64) -> Result<(), io::Error> {
    let o = io::stdout();
    let mut ol = o.lock();

    let il = io::stdin().lock();
    let mut taken = il.take(lmt);
    buf.clear();
    taken.read_to_end(buf)?;

    hdr_bytes2hdrs2least2iowtr(buf, BufWriter::new(&mut ol))?;

    ol.flush()
}
