import main from "../main";

export const mainColor = '#ff3e00';

export const drawObjects = (ctx: CanvasRenderingContext2D, objects, [width, height], debug = false) => {
    objects.forEach(obj => {
        ctx.beginPath();
        ctx.strokeStyle = mainColor;
        ctx.lineWidth = 2;

        const obj_y = height - obj.y;
        const orientation_y = obj.orientation_y * -1;
        const vel_y = obj.vel_y * -1;

        // rect
        if (obj.shape_param_2) {
            ctx.moveTo(obj.x, obj_y);
            ctx.rect(obj.x - obj.shape_param_1 / 2, obj_y - obj.shape_param_2 / 2, obj.shape_param_1, obj.shape_param_2);
        }
        // circle
        else {
            ctx.arc(obj.x, obj_y, obj.shape_param_1, 0, 2 * Math.PI);
        }
        ctx.stroke();

        if (debug) {
            // velocity
            drawLine(ctx, obj.x, obj_y, obj.x + obj.vel_x * 20, obj_y + vel_y * 20, 'red')
            // orientation
            drawLine(ctx, obj.x, obj_y, obj.x + obj.orientation_x * 20, obj_y + orientation_y * 20, 'blue')
            ctx.fillText(`[x: ${obj.x}, y: ${obj_y}]`, obj.x + 10, obj_y)
        }
    })
}

export const drawLine = (ctx, from_x, from_y, to_x, to_y, color) => {
    ctx.beginPath();
    ctx.moveTo(from_x, from_y);
    ctx.strokeStyle = color;
    ctx.lineTo(to_x, to_y);
    ctx.stroke();
}
