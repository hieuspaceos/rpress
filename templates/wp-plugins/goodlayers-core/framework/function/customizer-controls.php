<?php
	/*	
	*	Goodlayers Customizer Controls
	*	---------------------------------------------------------------------
	*	File which creates the customizer custom controls element
	*	---------------------------------------------------------------------
	*/

if( !class_exists('GDLR_Core_Customize_Description') ){
	class GDLR_Core_Customize_Description extends WP_Customize_Control{

		public $type = 'gdlr_core_description';		
		
		public function render_content(){
?>
	<span class="customize-control-title"><?php echo gdlr_core_escape_content($this->label); ?></span>
<?php
		}
	
	} // GDLR_Core_Customize_Description
} // class_exists

if( !class_exists('GDLR_Core_Customize_FontSlider_Control') ){
	class GDLR_Core_Customize_FontSlider_Control extends WP_Customize_Control{

		public $type = 'gdlr_core_fontslider';		
		
		public function render_content(){
?>
	<span class="customize-control-title"><?php echo esc_html($this->label); ?></span>
	<input type="text" class="gdlr-core-customizer-fontslider" value="<?php echo esc_attr($this->value()); ?>" <?php $this->link(); ?> <?php
		echo isset($this->choices['min'])? ' data-min-value="' . esc_attr($this->choices['min']) . '" ': '';
		echo isset($this->choices['max'])? ' data-max-value="' . esc_attr($this->choices['max']) . '" ': '';
		echo isset($this->choices['suffix'])? ' data-suffix="' . esc_attr($this->choices['suffix']) . '" ': '';
	?> />
<?php

		}
	
	} // GDLR_Core_Customize_FontSlider_Control
} // class_exists

if( !class_exists('GDLR_Core_Customize_Checkbox_Control') ){
	class GDLR_Core_Customize_Checkbox_Control extends WP_Customize_Control{
		
		// escape content from database
		static function sanitize_js_customizer_checkbox($input){
			return ($input == 'disable')? 0: 1; 
		}		
		
		// escape content with html
		static function sanitize_customizer_checkbox($input){
			return empty($input)? 'disable': 'enable';
		}		
		
		public $type = 'checkbox';		
		
		public function render_content(){
?>
	<span class="customize-control-title"><?php echo esc_html($this->label); ?></span>
	<label class="gdlr-core-customizer-checkbox-wrapper" >
		<input type="checkbox" value="enable" class="gdlr-core-customizer-checkbox" <?php checked($this->value(), 'enable'); ?> <?php $this->link(); ?> />
		<div class="gdlr-core-customizer-checkbox-appearance gdlr-core-noselect">
			<span class="gdlr-core-checkbox-button gdlr-core-on"><?php esc_html_e('On', 'goodlayers-core'); ?></span>
			<span class="gdlr-core-checkbox-separator"></span>
			<span class="gdlr-core-checkbox-button gdlr-core-off"><?php esc_html_e('Off', 'goodlayers-core'); ?></span>
		</div>
	</label>
<?php
		}
	
	} // GDLR_Core_Customize_Checkbox_Control
} // class_exists

if( !class_exists('GDLR_Core_Customize_RadioImage_Control') ){
	class GDLR_Core_Customize_RadioImage_Control extends WP_Customize_Control{		
		
		public $type = 'radio';		
		
		public function render_content(){
			$value = $this->value();
			if( empty($value) ){
				reset($this->choices);
				$value = key($this->choices);
			}
?>
<span class="customize-control-radioimage-title"><?php echo esc_html($this->label); ?></span>
<?php foreach( $this->choices as $option_key => $option_url ){ ?>
	<label class="gdlr-core-customizer-radio-wrapper" <?php  echo gdlr_core_esc_style(array(
			'max-width'=> empty($this->input_attrs['max-width'])? '': $this->input_attrs['max-width']
	)); ?> >
		<input class="gdlr-core-customizer-radioimage" type="radio" name="<?php echo esc_attr($this->id); ?>" value="<?php echo esc_attr($option_key); ?>" <?php checked($value, $option_key); ?> <?php $this->link(); ?> />
		<?php 
			if( !empty($this->input_attrs['type']) && $this->input_attrs['type'] == 'radioimage-frame' ){
				echo '<div class="gdlr-core-radioimage-frame" ></div>';	
			}else{
				echo '<div class="gdlr-core-radioimage-checked" ></div>';
			}
		?>
		<img src="<?php echo esc_url($option_url); ?>" alt="<?php echo esc_attr($option_key) ?>" />
	</label>
<?php } // foreach
	
		}
	
	} // GDLR_Core_Customize_RadioImage_Control
} // class_exists

if( !class_exists('GDLR_Core_Customize_Font_Control') ){
	class GDLR_Core_Customize_Font_Control extends WP_Customize_Control{
		
		public $type = 'select';
		
		public function render_content(){
?>
	<span class="customize-control-title"><?php echo esc_html($this->label); ?></span>
	<select <?php $this->link(); ?> >
	<?php
		global $gdlr_core_font_loader;
		if( empty($gdlr_core_font_loader) ){
			$gdlr_core_font_loader = new gdlr_core_font_loader();
		}
		
		if( !empty($this->value) ){
			echo gdlr_core_escape_content($gdlr_core_font_loader->get_option_list($this->value));
		}else{
			echo gdlr_core_escape_content($gdlr_core_font_loader->get_option_list(''));
		}	
	?>
	</select>
<?php
		} 

	} // GDLR_Core_Customize_Font_Control
} // class_exists