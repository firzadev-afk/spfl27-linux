// Offside banner (FL27): offside.lua (Sider) drives BannerTime/BannerAlpha/BannerTournament through offside_banner.addon64.
// Textures in reshade-shaders\Textures:
//   offside_banner.png    1024x256 shape mask (make_banner.py): R box, G logo tab, B text
//   offside_logos.png     2048x2048 competition emblems, emblem e in cell e of 16x16 (0: VAR badge), premultiplied (make_logos.py)
//   offside_logo_map.png  1024x4: row 0 tournament -> emblem, rows 1-3 per emblem: tab top, tab bottom, text colour
#include "ReShade.fxh"

uniform float BannerTime < hidden = true; > = -1.0;   // seconds since it appeared, below 0 = hidden
uniform float BannerAlpha < hidden = true; > = 0.0;
uniform float BannerTournament < hidden = true; > = 0.0;

uniform float2 BannerCentre < ui_type = "drag"; ui_min = 0.0; ui_max = 1.0; ui_step = 0.005;
    ui_label = "Centre (x, y)"; ui_tooltip = "Screen position of the banner's centre, 0..1."; > = float2(0.5, 0.86);
uniform float BannerHeight < ui_type = "drag"; ui_min = 0.02; ui_max = 0.3; ui_step = 0.005;
    ui_label = "Height"; ui_tooltip = "Banner height as a part of the screen height."; > = 0.09;
uniform float SweepTime < ui_type = "drag"; ui_min = 0.05; ui_max = 2.0; ui_step = 0.05;
    ui_label = "Sweep time (s)"; > = 0.45;
uniform bool ShowLogo < ui_label = "Competition logo"; > = true;

texture BannerTex < source = "offside_banner.png"; > { Width = 1024; Height = 256; Format = RGBA8; };
sampler BannerSampler { Texture = BannerTex; AddressU = CLAMP; AddressV = CLAMP; };
texture LogoTex < source = "offside_logos.png"; > { Width = 2048; Height = 2048; Format = RGBA8; MipLevels = 3; };
sampler LogoSampler { Texture = LogoTex; AddressU = CLAMP; AddressV = CLAMP; };
texture LogoMapTex < source = "offside_logo_map.png"; > { Width = 1024; Height = 4; Format = RGBA8; };
sampler LogoMapSampler { Texture = LogoMapTex; MinFilter = POINT; MagFilter = POINT; MipFilter = POINT; };

// banner layout in its 1024x256 image (make_banner.py)
static const float2 BOX_Y = float2(28.0, 228.0) / 256.0;
static const float2 TAB_CENTRE = float2(123.0 / 1024.0, 0.5);
static const float LOGO_CELL = 180.0;   // px of the banner image covered by one 128 px atlas cell

float4 BannerPS(float4 pos : SV_Position, float2 uv : TEXCOORD) : SV_Target
{
    float4 col = tex2D(ReShade::BackBuffer, uv);
    if (BannerTime < 0.0 || BannerAlpha <= 0.0)
        return col;
    // banner box on screen, 4:1 in pixels
    float2 size = float2(BannerHeight * 4.0 * BUFFER_HEIGHT * BUFFER_RCP_WIDTH, BannerHeight);
    float s = saturate(BannerTime / SweepTime);
    s = s * s * (3.0 - 2.0 * s);
    // slides in from the left while it is revealed left to right
    float2 corner = BannerCentre - size * 0.5 - float2((1.0 - s) * size.x * 0.12, 0.0);
    float2 bp = (uv - corner) / size;
    if (any(bp < 0.0) || any(bp > 1.0))
        return col;
    float3 mask = tex2D(BannerSampler, bp).rgb;

    // the competition: its emblem (0 = none: the VAR badge) and colours
    float4 entry = tex2Dfetch(LogoMapSampler, int2(int(BannerTournament + 0.5) % 1024, 0));
    int e = entry.a > 0.5 ? int(entry.r * 255.0 + 0.5) : 0;
    float3 top = tex2Dfetch(LogoMapSampler, int2(e, 1)).rgb;
    float3 bottom = tex2Dfetch(LogoMapSampler, int2(e, 2)).rgb;
    float3 ink = tex2Dfetch(LogoMapSampler, int2(e, 3)).rgb;

    float3 c = 1.0;
    c = lerp(c, lerp(top, bottom, saturate((bp.y - BOX_Y.x) / (BOX_Y.y - BOX_Y.x))), mask.g);
    c = lerp(c, ink, mask.b);
    if (ShowLogo)   // cell 0: the VAR badge, for a match without a competition
    {
        float2 lp = (bp - TAB_CENTRE) * float2(1024.0, 256.0) / LOGO_CELL + 0.5;
        if (all(lp >= 0.0) && all(lp <= 1.0))
        {
            float4 logo = tex2D(LogoSampler, (float2(e % 16, e / 16) + lp) / 16.0);
            c = logo.rgb + c * (1.0 - logo.a);
        }
    }
    float reveal = saturate((s * 1.08 - bp.x) / 0.08);   // soft edge on the sweep
    return float4(lerp(col.rgb, c, mask.r * reveal * BannerAlpha), col.a);
}

technique OffsideBanner < ui_label = "Offside banner (driven by Sider)"; enabled = true; >
{
    pass
    {
        VertexShader = PostProcessVS;
        PixelShader = BannerPS;
    }
}
