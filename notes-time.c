
// --------------------------------------------------------------------
// making sense of c time
// --------------------------------------------------------------------


#define __USE_MISC
#define time_t long int
#define __time64_t int // 8 bytes
#define SECSPERDAY 86400
#define SECS_PER_DAY SECSPERDAY
#define SECS_PER_HOUR (60 * 60)
#define NULL ((void *) 0)
#define __use_tzfile 0 // or 1


// --------------------------------------------------------------------

struct tm
{
  int tm_sec;			/* Seconds.	[0-60] (1 leap second) */
  int tm_min;			/* Minutes.	[0-59] */
  int tm_hour;			/* Hours.	[0-23] */
  int tm_mday;			/* Day.		[1-31] */
  int tm_mon;			/* Month.	[0-11] */
  int tm_year;			/* Year	- 1900.  */
  int tm_wday;			/* Day of week.	[0-6] */
  int tm_yday;			/* Days in year.[0-365]	*/
  int tm_isdst;			/* DST.		[-1/0/1]*/

# ifdef	__USE_MISC 
  long int tm_gmtoff;		/* Seconds east of UTC.  */
  const char *tm_zone;		/* Timezone abbreviation.  */
# else
  long int __tm_gmtoff;		/* Seconds east of UTC.  */
  const char *__tm_zone;	/* Timezone abbreviation.  */
# endif
};

struct tm _tmbuf;

// --------------------------------------------------------------------


typedef struct
  {
    const char *name;

    /* When to change.  */
    enum { J0, J1, M } type;	/* Interpretation of:  */
    unsigned short int m, n, d;	/* Month, week, day.  */
    int secs;			/* Time of day.  */

    int offset;			/* Seconds east of GMT (west if < 0).  */

    /* We cache the computed time of change for a
       given year so we don't have to recompute it.  */
    __time64_t change;	/* When to change to this zone.  */
    int computed_for;	/* Year above is computed for.  */
  } tz_rule;


/* tz_rules[0] is standard, tz_rules[1] is daylight.  */
static tz_rule tz_rules[2];


// --------------------------------------------------------------------

#define __isleap(year)	\
  ((year) % 4 == 0 && ((year) % 100 != 0 || (year) % 400 == 0))


// --------------------------------------------------------------------
/* How many days come before each month (0-12).  */
#ifndef _LIBC
static
#endif
const unsigned short int __mon_yday[2][13] =
  {
    /* Normal years.  */
    { 0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334, 365 },
    /* Leap years.  */
    { 0, 31, 60, 91, 121, 152, 182, 213, 244, 274, 305, 335, 366 }
  };

// --------------------------------------------------------------------

static void
compute_change (tz_rule *rule, int year)
{
  __time64_t t;

  if (year != -1 && rule->computed_for == year)
    /* Operations on times in 2 BC will be slower.  Oh well.  */
    return;

  /* First set T to January 1st, 0:00:00 GMT in YEAR.  */
  if (year > 1970)
    t = ((year - 1970) * 365
	 + /* Compute the number of leapdays between 1970 and YEAR
	      (exclusive).  There is a leapday every 4th year ...  */
	 + ((year - 1) / 4 - 1970 / 4)
	 /* ... except every 100th year ... */
	 - ((year - 1) / 100 - 1970 / 100)
	 /* ... but still every 400th year.  */
	 + ((year - 1) / 400 - 1970 / 400)) * SECSPERDAY;
  else
    t = 0;

  switch (rule->type)
    {
    case J1:
      /* Jn - Julian day, 1 == January 1, 60 == March 1 even in leap years.
	 In non-leap years, or if the day number is 59 or less, just
	 add SECSPERDAY times the day number-1 to the time of
	 January 1, midnight, to get the day.  */
      t += (rule->d - 1) * SECSPERDAY;
      if (rule->d >= 60 && __isleap (year))
	t += SECSPERDAY;
      break;

    case J0:
      /* n - Day of year.
	 Just add SECSPERDAY times the day number to the time of Jan 1st.  */
      t += rule->d * SECSPERDAY;
      break;

    case M:
      /* Mm.n.d - Nth "Dth day" of month M.  */
      {
	unsigned int i;
	int d, m1, yy0, yy1, yy2, dow;
	const unsigned short int *myday = &__mon_yday[__isleap (year)][rule->m];

	/* First add SECSPERDAY for each day in months before M.  */
	t += myday[-1] * SECSPERDAY;

	/* Use Zeller's Congruence to get day-of-week of first day of month. */
	m1 = (rule->m + 9) % 12 + 1;
	yy0 = (rule->m <= 2) ? (year - 1) : year;
	yy1 = yy0 / 100;
	yy2 = yy0 % 100;
	dow = ((26 * m1 - 2) / 10 + 1 + yy2 + yy2 / 4 + yy1 / 4 - 2 * yy1) % 7;
	if (dow < 0)
	  dow += 7;

	/* DOW is the day-of-week of the first day of the month.  Get the
	   day-of-month (zero-origin) of the first DOW day of the month.  */
	d = rule->d - dow;
	if (d < 0)
	  d += 7;
	for (i = 1; i < rule->n; ++i)
	  {
	    if (d + 7 >= (int) myday[0] - myday[-1])
	      break;
	    d += 7;
	  }

	/* D is the day-of-month (zero-origin) of the day we want.  */
	t += d * SECSPERDAY;
      }
      break;
    }

  /* T is now the Epoch-relative time of 0:00:00 GMT on the day we want.
     Just add the time of day and local offset from GMT, and we're done.  */

  rule->change = t - rule->offset + rule->secs;
  rule->computed_for = year;
}

void
__tz_compute (__time64_t timer, struct tm *tm, int use_localtime)
{
  compute_change (&tz_rules[0], 1900 + tm->tm_year);
  compute_change (&tz_rules[1], 1900 + tm->tm_year);

  if (use_localtime)
    {
      int isdst;

      /* We have to distinguish between northern and southern
	 hemisphere.  For the latter the daylight saving time
	 ends in the next year.  */
      if (__builtin_expect (tz_rules[0].change
			    > tz_rules[1].change, 0))
	isdst = (timer < tz_rules[1].change
		 || timer >= tz_rules[0].change);
      else
	isdst = (timer >= tz_rules[0].change
		 && timer < tz_rules[1].change);
      tm->tm_isdst = isdst;
      tm->tm_zone = __tzname[isdst];
      tm->tm_gmtoff = tz_rules[isdst].offset;
    }
}


// --------------------------------------------------------------------

#define DIV(a, b) ((a) / (b) - ((a) % (b) < 0))
#define LEAPS_THRU_END_OF(y) (DIV (y, 4) - DIV (y, 100) + DIV (y, 400))

// --------------------------------------------------------------------

int
__offtime (__time64_t t, long int offset, struct tm *tp)
// enter here with t = secs from unix (long int)
// enter here with offset = 0
// enter here with *tp as *_tmbuf = NULL
{
  __time64_t days, rem, y;
  const unsigned short int *ip;

  days = t / SECS_PER_DAY;
  rem = t % SECS_PER_DAY;
  rem += offset;
  while (rem < 0)
    {
      rem += SECS_PER_DAY;
      --days;
    }
  while (rem >= SECS_PER_DAY)
    {
      rem -= SECS_PER_DAY;
      ++days;
    }
  tp->tm_hour = rem / SECS_PER_HOUR;
  rem %= SECS_PER_HOUR;
  tp->tm_min = rem / 60;
  tp->tm_sec = rem % 60;
  /* January 1, 1970 was a Thursday.  */
  tp->tm_wday = (4 + days) % 7;
  if (tp->tm_wday < 0)
    tp->tm_wday += 7;

  y = 1970;

  while (days < 0 || days >= (__isleap (y) ? 366 : 365))
    {
      /* Guess a corrected year, assuming 365 days per year.  */
      __time64_t yg = y + days / 365 - (days % 365 < 0);

      /* Adjust DAYS and Y to match the guessed year.  */
      days -= ((yg - y) * 365 + LEAPS_THRU_END_OF (yg - 1) - LEAPS_THRU_END_OF (y - 1));
      y = yg;
    }
  tp->tm_year = y - 1900;
  if (tp->tm_year != y - 1900)
    {
      /* The year cannot be represented due to overflow.  */
      // __set_errno (EOVERFLOW);
      return 0;
    }
  tp->tm_yday = days;
  ip = __mon_yday[__isleap(y)];
  for (y = 11; days < (long int) ip[y]; --y)
    continue;
  days -= ip[y];
  tp->tm_mon = y;
  tp->tm_mday = days + 1;
  return 1;
}
// --------------------------------------------------------------------

struct tm *
__tz_convert (__time64_t timer, int use_localtime, struct tm *tp)
// timer = secs from unix
// use_localtime = 0
// *tp = pointer to NULL "tm" struct
{
  long int leap_correction;
  int leap_extra_secs;

  // __libc_lock_lock (tzset_lock);

  /* Update internal database according to current TZ setting.
     POSIX.1 8.3.7.2 says that localtime_r is not required to set tzname.
     This is a good idea since this allows at least a bit more parallelism.  */
  // tzset_internal (tp == &_tmbuf && use_localtime);

  if (__use_tzfile)
    // do not care
    return 0;
    // __tzfile_compute (timer, use_localtime, &leap_correction, &leap_extra_secs, tp);
  else
    {
      if (! __offtime (timer, 0, tp))
	tp = NULL;
      else
	__tz_compute (timer, tp, use_localtime);
      leap_correction = 0L;
      leap_extra_secs = 0;
    }

  // __libc_lock_unlock (tzset_lock);

  if (tp)
    {
      if (! use_localtime)
	{
	  tp->tm_isdst = 0;
	  tp->tm_zone = "GMT";
	  tp->tm_gmtoff = 0L;
	}

      if (__offtime (timer, tp->tm_gmtoff - leap_correction, tp))
        tp->tm_sec += leap_extra_secs;
      else
	tp = NULL;
    }

  return tp;
}

// --------------------------------------------------------------------

struct tm *
__gmtime64 (const __time64_t *t)
// *t is pointer to long int containing secs from unix epoch
{
  return __tz_convert (*t, 0, &_tmbuf);
}
// struct tm *
// __gmtime64_r (const __time64_t *t, struct tm *tp)
// {
//   return __tz_convert (*t, 0, tp);
// }

// --------------------------------------------------------------------

#include <stdio.h>


int main() {
    printf("Hello, World!\n");

    // // time_t = long int
    // time_t now; 
    // time(&now);
    //
    // printf("from %ld :\n", now);
    //
    // struct tm* ptime;
    // ptime = gmtime(&now);
}
