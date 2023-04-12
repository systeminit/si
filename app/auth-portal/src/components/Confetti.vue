// adapted from https://codepen.io/iprodev/pen/azpWBr

<template>
  <div
    v-if="active"
    ref="canvasParentRef"
    class="fixed inset-0 z-[1000] pointer-events-none"
  >
    <canvas
      id="confetti"
      ref="canvasRef"
      class="absolute w-full h-full left-0 top-0"
    />
  </div>
</template>

<script setup lang="ts">
/* eslint-disable */
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";

const canvasRef = ref<HTMLCanvasElement>();
const canvasParentRef = ref<HTMLDivElement>();

// Math shorthands
const PI = Math.PI;
const sqrt = Math.sqrt;
const round = Math.round;
const random = Math.random;
const cos = Math.cos;
const sin = Math.sin;

// Local WindowAnimationTiming interface
let rAF = window.requestAnimationFrame;
// @ts-ignore
let cAF = window.cancelAnimationFrame || window.cancelRequestAnimationFrame;
const _now = Date.now || (() => new Date().getTime());

// Local WindowAnimationTiming interface polyfill
(function (w) {
  /**
   * Fallback implementation.
   */
  let prev = _now();
  function fallback(fn: any) {
    const curr = _now();
    const ms = Math.max(0, 16 - (curr - prev));
    const req = setTimeout(fn, ms);
    prev = curr;
    return req;
  }

  /**
   * Cancel.
   */
  
  const cancel =
    // @ts-ignore
    w.cancelAnimationFrame || w.webkitCancelAnimationFrame || w.clearTimeout; 

  // @ts-ignore
  rAF = w.requestAnimationFrame || w.webkitRequestAnimationFrame || fallback;

  cAF = function (id) {
    cancel.call(w, id);
  };
})(window);


const speed = 50;
const duration = 1.0 / speed;
const confettiRibbonCount = 10;
const ribbonPaperCount = 30;
const ribbonPaperDist = 8.0;
const ribbonPaperThick = 8.0;
const confettiPaperCount = 100;
const DEG_TO_RAD = PI / 180;
const RAD_TO_DEG = 180 / PI;
const colors = [
  ["#df0049", "#660671"],
  ["#00e857", "#005291"],
  ["#2bebbc", "#05798a"],
  ["#ffd200", "#b06c00"],
];

class Vector2 {
  constructor(public x: number, public y: number) {}
  Length() {
    return sqrt(this.SqrLength());
  };
  SqrLength() {
    return this.x * this.x + this.y * this.y;
  };
  Add(_vec: any) {
    this.x += _vec.x;
    this.y += _vec.y;
  };
  Sub(_vec: any) {
    this.x -= _vec.x;
    this.y -= _vec.y;
  };
  Div(_f: any) {
    this.x /= _f;
    this.y /= _f;
  };
  Mul(_f: any) {
    this.x *= _f;
    this.y *= _f;
  };
  Normalize() {
    const sqrLen = this.SqrLength();
    if (sqrLen != 0) {
      const factor = 1.0 / sqrt(sqrLen);
      this.x *= factor;
      this.y *= factor;
    }
  };
  Normalized() {
    const sqrLen = this.SqrLength();
    if (sqrLen != 0) {
      const factor = 1.0 / sqrt(sqrLen);
      return new Vector2(this.x * factor, this.y * factor);
    }
    return new Vector2(0, 0);
  };

  static Lerp(_vec0: Vector2, _vec1: Vector2, _t: number) {
    return new Vector2(
      (_vec1.x - _vec0.x) * _t + _vec0.x,
      (_vec1.y - _vec0.y) * _t + _vec0.y,
    );
  };

static Distance(_vec0: Vector2, _vec1: Vector2) {
  return sqrt(Vector2.SqrDistance(_vec0, _vec1));
};
static SqrDistance(_vec0: Vector2, _vec1: Vector2) {
  const x = _vec0.x - _vec1.x;
  const y = _vec0.y - _vec1.y;
  return x * x + y * y/* + z * z*/;
};
static Scale(_vec0: Vector2, _vec1: Vector2) {
  return new Vector2(_vec0.x * _vec1.x, _vec0.y * _vec1.y);
};
static Min(_vec0: Vector2, _vec1: Vector2) {
  return new Vector2(Math.min(_vec0.x, _vec1.x), Math.min(_vec0.y, _vec1.y));
};
static Max(_vec0: Vector2, _vec1: Vector2) {
  return new Vector2(Math.max(_vec0.x, _vec1.x), Math.max(_vec0.y, _vec1.y));
};
static ClampMagnitude(_vec0: Vector2, _len: number) {
  const vecNorm = _vec0.Normalized();
  return new Vector2(vecNorm.x * _len, vecNorm.y * _len);
};
static Sub(_vec0: Vector2, _vec1: Vector2) {
  return new Vector2(_vec0.x - _vec1.x, _vec0.y - _vec1.y/*, _vec0.z - _vec1.z*/);
};

}


class EulerMass {
  public position: Vector2;
  public force: Vector2;
  public velocity: Vector2;

  constructor(_x: number, _y: number, public mass: number, public drag: number) {
    this.position = new Vector2(_x, _y);
    this.force = new Vector2(0, 0);
    this.velocity = new Vector2(0, 0);
  }
  

  AddForce(_f:Vector2) {
    this.force.Add(_f);
  };
  Integrate(_dt:number) {
    const acc = this.CurrentForce(this.position);
    acc.Div(this.mass);
    const posDelta = new Vector2(this.velocity.x, this.velocity.y);
    posDelta.Mul(_dt);
    this.position.Add(posDelta);
    acc.Mul(_dt);
    this.velocity.Add(acc);
    this.force = new Vector2(0, 0);
  };
  CurrentForce(_pos:Vector2/*, _vel:Vector2*/) {
    const totalForce = new Vector2(this.force.x, this.force.y);
    const speed = this.velocity.Length();
    const dragVel = new Vector2(this.velocity.x, this.velocity.y);
    dragVel.Mul(this.drag * this.mass * speed);
    totalForce.Sub(dragVel);
    return totalForce;
  };
}

class ConfettiPaper {
  public pos: Vector2;
  public rotationSpeed: number;
  public angle: number;
  public rotation: number;
  public cosA: number;
  public size: number;
  public oscillationSpeed: number;
  public xSpeed: number;
  public ySpeed: number;
  public corners: Vector2[];
  public time: number;
  public frontColor: string;
  public backColor: string;


  constructor(_x:number, _y:number) {
    this.pos = new Vector2(_x, _y);
    this.rotationSpeed = random() * 600 + 800;
    this.angle = DEG_TO_RAD * random() * 360;
    this.rotation = DEG_TO_RAD * random() * 360;
    this.cosA = 1.0;
    this.size = 5.0;
    this.oscillationSpeed = random() * 1.5 + 0.5;
    this.xSpeed = 40.0;
    this.ySpeed = random() * 60 + 50.0;
    this.corners = [];
    this.time = random();
    const ci = round(random() * (colors.length - 1));
    this.frontColor = colors[ci][0];
    this.backColor = colors[ci][1];
    for (let i = 0; i < 4; i++) {
      const dx = cos(this.angle + DEG_TO_RAD * (i * 90 + 45));
      const dy = sin(this.angle + DEG_TO_RAD * (i * 90 + 45));
      this.corners[i] = new Vector2(dx, dy);
    }
  }
  Update(_dt:number) {
    if (this.pos.y > canvasHeight.value) {
      if (!props.noLoop) this.Reset();
      return;
    }
    
    this.time += _dt;
    this.rotation += this.rotationSpeed * _dt;
    this.cosA = cos(DEG_TO_RAD * this.rotation);
    this.pos.x += cos(this.time * this.oscillationSpeed) * this.xSpeed * _dt;
    this.pos.y += this.ySpeed * _dt;
  }
  Reset() {
    this.pos.x = random() * canvasWidth.value;
    this.pos.y = 0;
  }
  Draw(_g: CanvasRenderingContext2D) {
    if (this.cosA > 0) {
      _g.fillStyle = this.frontColor;
    } else {
      _g.fillStyle = this.backColor;
    }
    _g.beginPath();
    _g.moveTo(
      (this.pos.x + this.corners[0].x * this.size) * retinaRatio.value,
      (this.pos.y + this.corners[0].y * this.size * this.cosA) * retinaRatio.value,
    );
    for (let i = 1; i < 4; i++) {
      _g.lineTo(
        (this.pos.x + this.corners[i].x * this.size) * retinaRatio.value,
        (this.pos.y + this.corners[i].y * this.size * this.cosA) * retinaRatio.value,
      );
    }
    _g.closePath();
    _g.fill();
  };
}

class ConfettiRibbon {
  public particles: EulerMass[];
  public frontColor: string;
  public backColor: string;
  public xOff: number;
  public yOff: number;
  public position: Vector2;
  public prevPosition: Vector2;
  public velocityInherit: number;
  public time: number;
  public oscillationSpeed: number;
  public oscillationDistance: number;
  public ySpeed: number;

  constructor(
    _x: number,
    _y: number,
    public particleCount: number,
    public particleDist: number,
    _thickness: number,
    _angle: number,
    public particleMass: number,
    public particleDrag: number,
  ) {
    
    this.particles = [];
    const ci = round(random() * (colors.length - 1));
    this.frontColor = colors[ci][0];
    this.backColor = colors[ci][1];
    this.xOff = cos(DEG_TO_RAD * _angle) * _thickness;
    this.yOff = sin(DEG_TO_RAD * _angle) * _thickness;
    this.position = new Vector2(_x, _y);
    this.prevPosition = new Vector2(_x, _y);
    this.velocityInherit = random() * 2 + 4;
    this.time = random() * 100;
    this.oscillationSpeed = random() * 2 + 2;
    this.oscillationDistance = random() * 40 + 40;
    this.ySpeed = random() * 40 + 80;
    for (let i = 0; i < this.particleCount; i++) {
      this.particles[i] = new EulerMass(
        _x,
        _y - i * this.particleDist,
        this.particleMass,
        this.particleDrag,
      );
    }
  }
  Update(_dt: number) {
    if (
      this.position.y >
      canvasHeight.value + this.particleDist * this.particleCount
    ) {
      if (!props.noLoop) this.Reset();
      return;
    }

    let i = 0;
    this.time += _dt * this.oscillationSpeed;
    this.position.y += this.ySpeed * _dt;
    this.position.x += cos(this.time) * this.oscillationDistance * _dt;
    this.particles[0].position = this.position;
    const dX = this.prevPosition.x - this.position.x;
    const dY = this.prevPosition.y - this.position.y;
    const delta = sqrt(dX * dX + dY * dY);
    this.prevPosition = new Vector2(this.position.x, this.position.y);
    for (i = 1; i < this.particleCount; i++) {
      const dirP = Vector2.Sub(
        this.particles[i - 1].position,
        this.particles[i].position,
      );
      dirP.Normalize();
      dirP.Mul((delta / _dt) * this.velocityInherit);
      this.particles[i].AddForce(dirP);
    }
    for (i = 1; i < this.particleCount; i++) {
      this.particles[i].Integrate(_dt);
    }
    for (i = 1; i < this.particleCount; i++) {
      const rp2 = new Vector2(
        this.particles[i].position.x,
        this.particles[i].position.y,
      );
      rp2.Sub(this.particles[i - 1].position);
      rp2.Normalize();
      rp2.Mul(this.particleDist);
      rp2.Add(this.particles[i - 1].position);
      this.particles[i].position = rp2;
    }
  };
  Reset() {
    this.position.y = -random() * canvasHeight.value;
    this.position.x = random() * canvasWidth.value;
    this.prevPosition = new Vector2(this.position.x, this.position.y);
    this.velocityInherit = random() * 2 + 4;
    this.time = random() * 100;
    this.oscillationSpeed = random() * 2.0 + 1.5;
    this.oscillationDistance = random() * 40 + 40;
    this.ySpeed = random() * 40 + 80;
    const ci = round(random() * (colors.length - 1));
    this.frontColor = colors[ci][0];
    this.backColor = colors[ci][1];
    this.particles = [];
    for (let i = 0; i < this.particleCount; i++) {
      this.particles[i] = new EulerMass(
        this.position.x,
        this.position.y - i * this.particleDist,
        this.particleMass,
        this.particleDrag,
      );
    }
  };
  Draw(_g: CanvasRenderingContext2D) {
    const retina = retinaRatio.value;
    for (let i = 0; i < this.particleCount - 1; i++) {
      const p0 = new Vector2(
        this.particles[i].position.x + this.xOff,
        this.particles[i].position.y + this.yOff,
      );
      const p1 = new Vector2(
        this.particles[i + 1].position.x + this.xOff,
        this.particles[i + 1].position.y + this.yOff,
      );
      if (
        this.Side(
          this.particles[i].position.x,
          this.particles[i].position.y,
          this.particles[i + 1].position.x,
          this.particles[i + 1].position.y,
          p1.x,
          p1.y,
        ) < 0
      ) {
        _g.fillStyle = this.frontColor;
        _g.strokeStyle = this.frontColor;
      } else {
        _g.fillStyle = this.backColor;
        _g.strokeStyle = this.backColor;
      }
      if (i == 0) {
        _g.beginPath();
        _g.moveTo(
          this.particles[i].position.x * retina,
          this.particles[i].position.y * retina,
        );
        _g.lineTo(
          this.particles[i + 1].position.x * retina,
          this.particles[i + 1].position.y * retina,
        );
        _g.lineTo(
          (this.particles[i + 1].position.x + p1.x) * 0.5 * retina,
          (this.particles[i + 1].position.y + p1.y) * 0.5 * retina,
        );
        _g.closePath();
        _g.stroke();
        _g.fill();
        _g.beginPath();
        _g.moveTo(p1.x * retina, p1.y * retina);
        _g.lineTo(p0.x * retina, p0.y * retina);
        _g.lineTo(
          (this.particles[i + 1].position.x + p1.x) * 0.5 * retina,
          (this.particles[i + 1].position.y + p1.y) * 0.5 * retina,
        );
        _g.closePath();
        _g.stroke();
        _g.fill();
      } else if (i == this.particleCount - 2) {
        _g.beginPath();
        _g.moveTo(
          this.particles[i].position.x * retina,
          this.particles[i].position.y * retina,
        );
        _g.lineTo(
          this.particles[i + 1].position.x * retina,
          this.particles[i + 1].position.y * retina,
        );
        _g.lineTo(
          (this.particles[i].position.x + p0.x) * 0.5 * retina,
          (this.particles[i].position.y + p0.y) * 0.5 * retina,
        );
        _g.closePath();
        _g.stroke();
        _g.fill();
        _g.beginPath();
        _g.moveTo(p1.x * retina, p1.y * retina);
        _g.lineTo(p0.x * retina, p0.y * retina);
        _g.lineTo(
          (this.particles[i].position.x + p0.x) * 0.5 * retina,
          (this.particles[i].position.y + p0.y) * 0.5 * retina,
        );
        _g.closePath();
        _g.stroke();
        _g.fill();
      } else {
        _g.beginPath();
        _g.moveTo(
          this.particles[i].position.x * retina,
          this.particles[i].position.y * retina,
        );
        _g.lineTo(
          this.particles[i + 1].position.x * retina,
          this.particles[i + 1].position.y * retina,
        );
        _g.lineTo(p1.x * retina, p1.y * retina);
        _g.lineTo(p0.x * retina, p0.y * retina);
        _g.closePath();
        _g.stroke();
        _g.fill();
      }
    }
  };
  Side(x1: number, y1: number, x2: number, y2: number, x3: number, y3: number) {
    return (x1 - x2) * (y3 - y2) - (y1 - y2) * (x3 - x2);
  };

}

// NOTE - some awkward stuff here adapting from pure js code to TS/vue

const retinaRatio = ref(window.devicePixelRatio);

const canvasElWidth = ref(1);
const canvasElHeight = ref(1);
const canvasWidth = computed(() => canvasElWidth.value * retinaRatio.value);
const canvasHeight = computed(() => canvasElHeight.value * retinaRatio.value);

let canvasContext: CanvasRenderingContext2D;

let confettiRibbons: ConfettiRibbon[] = [];
let confettiPapers: ConfettiPaper[] = [];

function windowResizeHandler() {
  if (!canvasParentRef.value || !canvasRef.value) return; 
  retinaRatio.value = window.devicePixelRatio;
  canvasElWidth.value = canvasParentRef.value.offsetWidth;
  canvasElHeight.value = canvasParentRef.value.offsetHeight;
  // set width/height on actual canvas el
  canvasRef.value.width = canvasWidth.value
  canvasRef.value.height = canvasHeight.value
} 

let interval: number | null = null;
let running = false;


function reset() {
  if (!canvasRef.value) return;

  windowResizeHandler();

  canvasContext = canvasRef.value.getContext("2d")!;
  let i = 0;
  for (i = 0; i < confettiRibbonCount; i++) {    
    confettiRibbons[i] = new ConfettiRibbon(
      random() * canvasElWidth.value,
      -random() * canvasElHeight.value,
      ribbonPaperCount,
      ribbonPaperDist,
      ribbonPaperThick,
      45,
      1,
      0.05,
    );
  }
  for (i = 0; i < confettiPaperCount; i++) {
    confettiPapers[i] = new ConfettiPaper(
      random() * canvasElWidth.value,
      props.startTop ?
        -random() * canvasElHeight.value
        : random() * canvasElHeight.value,
    );
  }
  play();
}

function play() {
  update();
}
function pause() {
  cAF(interval!);
}
function stop() {
  cAF(interval!);
  running = false;
}
function update() {
  canvasContext.clearRect(0, 0, canvasWidth.value, canvasHeight.value);
  let i;
  for (i = 0; i < confettiPaperCount; i++) {
    confettiPapers[i].Update(duration);
    confettiPapers[i].Draw(canvasContext);
  }
  for (i = 0; i < confettiRibbonCount; i++) {
    confettiRibbons[i].Update(duration);
    confettiRibbons[i].Draw(canvasContext);
  }
  interval = rAF(update);
}


const props = defineProps({
  active: Boolean,
  startTop: Boolean,
  noLoop: Boolean,
});

watch(
  () => props.active,
  () => {
    if (props.active) {
      // use nextTick because the div/canvas are not in the dom yet
      nextTick(reset);
    } else {
      stop();
    }
  },
);

onMounted(() => {
  windowResizeHandler();
  window.addEventListener("resize", windowResizeHandler);
  if (props.active) reset();
});
onBeforeUnmount(() => {
  window.removeEventListener("resize", windowResizeHandler);
})
</script>
